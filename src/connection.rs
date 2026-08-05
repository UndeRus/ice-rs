//! Соединение Ice: один сокет, много запросов в полёте.
//!
//! Ожидание ответа — `oneshot`, который заполняет reader-таск. Ни опроса, ни
//! `sleep`.
//!
//! Раньше ответы складывались в `Vec<MessageType>`, а вызывающий раз в
//! миллисекунду просматривал его линейным поиском. Следствий было три: воркер
//! просыпался вхолостую на каждом вызове; смерть соединения замечалась только по
//! истечении 30-секундного таймаута; и очередь росла без ограничения, потому что
//! удалялись из неё только совпавшие ответы.

use crate::encoding::{FromBytes, ToBytes};
use crate::errors::ProtocolError;
use crate::protocol::{Header, RawReply, RequestData};
use crate::transport::Transport;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

type BoxTransport = Box<dyn Transport + Send + Sync + Unpin>;
type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Верхняя граница размера кадра (аналог `Ice.MessageSizeMax`).
pub const MAX_MESSAGE_SIZE: i32 = 2 * 1024 * 1024;

/// Ожидающие ответа — либо соединение уже мёртво.
enum Pending {
    Live(HashMap<i32, oneshot::Sender<RawReply>>),
    /// Причина смерти. Новые запросы получают её сразу, не дожидаясь таймаута.
    Dead(String),
}

struct Shared {
    pending: std::sync::Mutex<Pending>,
    /// Пик одновременных запросов. Нужен, чтобы конкурентность можно было
    /// проверить фактом, а не замером времени: на локальном сервере вызовы
    /// быстрее шума таймера.
    max_in_flight: AtomicUsize,
}

impl Shared {
    /// Регистрирует ожидание. Делать это **до** записи кадра: иначе быстрый
    /// ответ обгонит вставку, reader не найдёт получателя и выбросит кадр, а
    /// вызов повиснет до таймаута.
    fn register(&self, id: i32, tx: oneshot::Sender<RawReply>) -> Result<(), BoxError> {
        let mut guard = self.pending.lock().unwrap();
        match &mut *guard {
            Pending::Live(map) => {
                map.insert(id, tx);
                let n = map.len();
                self.max_in_flight.fetch_max(n, Ordering::Relaxed);
                Ok(())
            }
            Pending::Dead(reason) => Err(Box::new(ProtocolError::new(&format!(
                "Connection is dead: {}",
                reason
            )))),
        }
    }

    fn take(&self, id: i32) -> Option<oneshot::Sender<RawReply>> {
        match &mut *self.pending.lock().unwrap() {
            Pending::Live(map) => map.remove(&id),
            Pending::Dead(_) => None,
        }
    }

    /// Снимает регистрацию. Вызывается из `Drop` гарда: если future вызывающего
    /// отменили (таймаут, `select!`), запись иначе осталась бы навсегда.
    fn forget(&self, id: i32) {
        if let Pending::Live(map) = &mut *self.pending.lock().unwrap() {
            map.remove(&id);
        }
    }

    /// Помечает соединение мёртвым и роняет всех ожидающих.
    ///
    /// Дроп карты дропает все `Sender`, поэтому каждый `rx.await` немедленно
    /// возвращает ошибку — вместо того чтобы досиживать до таймаута.
    fn die(&self, reason: String) {
        let mut guard = self.pending.lock().unwrap();
        if let Pending::Live(_) = &*guard {
            *guard = Pending::Dead(reason);
        }
    }

    fn death_reason(&self) -> Option<String> {
        match &*self.pending.lock().unwrap() {
            Pending::Dead(r) => Some(r.clone()),
            Pending::Live(_) => None,
        }
    }
}

/// Снимает регистрацию, если ждать перестали.
struct PendingGuard<'a> {
    shared: &'a Shared,
    id: i32,
}

impl Drop for PendingGuard<'_> {
    fn drop(&mut self) {
        // При успехе запись уже забрал reader, и это no-op.
        self.shared.forget(self.id);
    }
}

/// Соединение с Ice-пиром.
pub struct Connection {
    shared: Arc<Shared>,
    /// Мьютекс держится только на время записи одного кадра — **не** через
    /// ожидание ответа. Именно это и даёт конкурентность.
    write: tokio::sync::Mutex<WriteHalf<BoxTransport>>,
    next_id: AtomicI32,
    reader: std::sync::Mutex<Option<JoinHandle<()>>>,
    peer: String,
    secure: bool,
}

impl Connection {
    /// Устанавливает соединение и дожидается рукопожатия.
    ///
    /// `ValidateConnection` читается здесь же, до запуска reader'а: он не несёт
    /// request_id, поэтому коррелировать его нечем — а по протоколу это первый
    /// кадр от сервера. Так в reader'е не появляется особого случая, а наружу
    /// выдаётся уже валидированное соединение.
    pub async fn connect(stream: BoxTransport, peer: &str) -> Result<Arc<Connection>, BoxError> {
        let secure = stream.transport_type() == "ssl";
        let (mut rx, tx) = tokio::io::split(stream);

        let frame = read_ice_message(&mut rx).await?;
        let mut read = 0i32;
        let header = Header::from_bytes(&frame, &mut read)?;
        if header.message_type != 3 {
            return Err(Box::new(ProtocolError::new(&format!(
                "Ice: expected ValidateConnection, got message type {}",
                header.message_type
            ))));
        }

        let shared = Arc::new(Shared {
            pending: std::sync::Mutex::new(Pending::Live(HashMap::new())),
            max_in_flight: AtomicUsize::new(0),
        });

        // Reader держит только `Arc<Shared>`, не `Arc<Connection>` — иначе
        // соединение никогда бы не дропалось.
        let reader_shared = shared.clone();
        let reader_peer = String::from(peer);
        let reader = tokio::spawn(async move {
            let reason = reader_loop(rx, &reader_shared).await;
            let _ = &reader_peer;
            reader_shared.die(reason);
        });

        Ok(Arc::new(Connection {
            shared,
            write: tokio::sync::Mutex::new(tx),
            // Ice: request_id 0 означает oneway, поэтому twoway начинаем с 1.
            next_id: AtomicI32::new(1),
            reader: std::sync::Mutex::new(Some(reader)),
            peer: String::from(peer),
            secure,
        }))
    }

    pub fn peer(&self) -> &str {
        &self.peer
    }

    pub fn is_secure(&self) -> bool {
        self.secure
    }

    pub fn is_alive(&self) -> bool {
        self.shared.death_reason().is_none()
    }

    /// Сколько запросов ждут ответа прямо сейчас.
    pub fn in_flight(&self) -> usize {
        match &*self.shared.pending.lock().unwrap() {
            Pending::Live(map) => map.len(),
            Pending::Dead(_) => 0,
        }
    }

    /// Пик одновременных запросов за время жизни соединения.
    ///
    /// Больше единицы означает, что мультиплексирование действительно работает.
    pub fn max_in_flight(&self) -> usize {
        self.shared.max_in_flight.load(Ordering::Relaxed)
    }

    /// Следующий request_id. Уникален в пределах соединения, а не прокси:
    /// несколько прокси делят один сокет и не должны пересекаться по id.
    pub fn next_request_id(&self) -> i32 {
        loop {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            // При переполнении не выдаём 0 (маркер oneway) и отрицательные.
            if id > 0 {
                return id;
            }
            self.next_id.store(1, Ordering::Relaxed);
        }
    }

    /// Отправляет запрос и ждёт ответа.
    pub async fn invoke(
        &self,
        request: &RequestData,
        timeout: Duration,
    ) -> Result<RawReply, BoxError> {
        let id = request.request_id;
        let (tx, rx) = oneshot::channel();

        // Порядок принципиален: сначала регистрация, потом запись.
        self.shared.register(id, tx)?;
        let _guard = PendingGuard {
            shared: &self.shared,
            id,
        };

        self.write_request(request).await?;

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(reply)) => Ok(reply),
            // Sender дропнут — значит соединение умерло.
            Ok(Err(_)) => Err(self.death_error()),
            Err(_) => Err(Box::new(ProtocolError::new(&format!(
                "Timeout after {:?} waiting for reply to request {} from {}",
                timeout, id, self.peer
            )))),
        }
    }

    /// Oneway-вызов: ответа не ждём и ничего не регистрируем.
    pub async fn invoke_oneway(&self, request: &RequestData) -> Result<(), BoxError> {
        self.write_request(request).await
    }

    async fn write_request(&self, request: &RequestData) -> Result<(), BoxError> {
        let body = request.to_bytes()?;
        let header = Header::new(0, 14 + body.len() as i32);
        let mut bytes = header.to_bytes()?;
        bytes.extend(body);

        let mut w = self.write.lock().await;
        w.write_all(&bytes).await?;
        w.flush().await?;
        Ok(())
    }

    fn death_error(&self) -> BoxError {
        let reason = self
            .shared
            .death_reason()
            .unwrap_or_else(|| String::from("connection closed"));
        Box::new(ProtocolError::new(&format!(
            "Connection to {} died: {}",
            self.peer, reason
        )))
    }

    /// Best-effort `CloseConnection`. Ошибку глушим: соединение всё равно
    /// закрывается.
    pub async fn close(&self) {
        if let Ok(bytes) = Header::new(4, 14).to_bytes() {
            let mut w = self.write.lock().await;
            let _ = w.write_all(&bytes).await;
            let _ = w.flush().await;
        }
        self.shared.die(String::from("closed locally"));
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Раньше здесь были `block_in_place` + `block_on`: первое паникует вне
        // multi-thread рантайма, второе — блокирующий вызов в `Drop`. Теперь
        // просто останавливаем reader; сокет закроется вместе с половинками.
        if let Some(h) = self.reader.lock().unwrap().take() {
            h.abort();
        }
    }
}

/// Читает кадры и будит ожидающих. Возвращает причину завершения.
async fn reader_loop(mut rx: ReadHalf<BoxTransport>, shared: &Arc<Shared>) -> String {
    loop {
        let frame = match read_ice_message(&mut rx).await {
            Ok(f) => f,
            Err(e) => return format!("read failed: {}", e),
        };
        let mut read = 0i32;
        let header = match Header::from_bytes(&frame, &mut read) {
            Ok(h) => h,
            Err(e) => return format!("bad header: {}", e),
        };

        match header.message_type {
            2 => match RawReply::from_bytes(&frame[read as usize..], &mut read) {
                Ok(reply) => {
                    if let Some(tx) = shared.take(reply.request_id) {
                        // Получатель мог уйти по таймауту — тогда send вернёт
                        // Err, и это нормально.
                        let _ = tx.send(reply);
                    }
                }
                // Битый ответ роняет только этот кадр, не соединение.
                Err(_) => continue,
            },
            // Входящий запрос: bidirectional не поддерживается, но кадр
            // проглатываем, а не убиваем соединение. Раньше это его убивало.
            0 => continue,
            3 => continue,
            4 => return String::from("peer sent CloseConnection"),
            // 1/6/7 — batch/compressed, не поддерживаем; кадр уже вычитан целиком.
            _ => continue,
        }
    }
}

/// Один полный кадр Ice: длина берётся из `Header.message_size`.
pub async fn read_ice_message(rx: &mut ReadHalf<BoxTransport>) -> Result<Vec<u8>, BoxError> {
    let mut hdr = [0u8; 14];
    rx.read_exact(&mut hdr).await?;
    let mut done = 0i32;
    let header = Header::from_bytes(&hdr, &mut done)?;
    // Проверять ДО приведения к usize: отрицательный i32 превращается в ~1.8e19,
    // проходит проверку `< 14` и уходит в аллокацию.
    if header.message_size < 14 || header.message_size > MAX_MESSAGE_SIZE {
        return Err(Box::new(ProtocolError::new(&format!(
            "Ice: implausible message_size {} (allowed 14..={})",
            header.message_size, MAX_MESSAGE_SIZE
        ))));
    }
    let total = header.message_size as usize;
    let mut buf = vec![0u8; total];
    buf[..14].copy_from_slice(&hdr);
    if total > 14 {
        rx.read_exact(&mut buf[14..]).await?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared() -> Arc<Shared> {
        Arc::new(Shared {
            pending: std::sync::Mutex::new(Pending::Live(HashMap::new())),
            max_in_flight: AtomicUsize::new(0),
        })
    }

    fn reply(id: i32) -> RawReply {
        RawReply {
            request_id: id,
            status: 0,
            payload: vec![],
        }
    }

    #[tokio::test]
    async fn reply_wakes_exactly_the_right_waiter() {
        let s = shared();
        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        s.register(1, tx1).unwrap();
        s.register(2, tx2).unwrap();

        s.take(2).unwrap().send(reply(2)).unwrap();
        assert_eq!(2, rx2.await.unwrap().request_id);
        // Первый всё ещё ждёт.
        assert!(
            tokio::time::timeout(Duration::from_millis(20), rx1)
                .await
                .is_err()
        );
    }

    /// Смерть соединения должна будить всех немедленно, а не оставлять их
    /// досиживать до таймаута.
    #[tokio::test]
    async fn death_wakes_all_waiters_at_once() {
        let s = shared();
        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        s.register(1, tx1).unwrap();
        s.register(2, tx2).unwrap();

        s.die(String::from("сокет закрылся"));

        // Никаких ожиданий: Sender'ы дропнуты вместе с картой.
        assert!(rx1.await.is_err());
        assert!(rx2.await.is_err());
        assert_eq!(Some(String::from("сокет закрылся")), s.death_reason());
    }

    /// На мёртвом соединении новый запрос должен падать сразу.
    #[tokio::test]
    async fn registering_on_a_dead_connection_fails_immediately() {
        let s = shared();
        s.die(String::from("уже мертво"));
        let (tx, _rx) = oneshot::channel();
        let err = s.register(1, tx).unwrap_err().to_string();
        assert!(err.contains("уже мертво"), "{}", err);
    }

    /// Отменённое ожидание не должно оставлять запись в карте: иначе
    /// долгоживущее соединение медленно течёт.
    #[test]
    fn cancelled_wait_leaves_no_entry() {
        let s = shared();
        {
            let (tx, _rx) = oneshot::channel();
            s.register(7, tx).unwrap();
            let _guard = PendingGuard {
                shared: &s,
                id: 7,
            };
            assert!(matches!(&*s.pending.lock().unwrap(),
                Pending::Live(m) if m.contains_key(&7)));
        }
        assert!(
            matches!(&*s.pending.lock().unwrap(), Pending::Live(m) if m.is_empty()),
            "гард обязан снять регистрацию"
        );
    }

    /// Ответ, которого никто не ждёт, не должен ничего ломать.
    #[test]
    fn unknown_reply_id_is_harmless() {
        let s = shared();
        assert!(s.take(42).is_none());
    }
}
