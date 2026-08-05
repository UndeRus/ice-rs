//! Подключение и уровень `Meta`.

use crate::error::{from_wire, Error, FaultContext, Result};
use crate::endpoint::{Endpoint, IntoEndpoint, TlsConfig};
use crate::ids::ServerId;
use crate::model::{DbState, Version};
use crate::server::VirtualServer;
use ice_rs::communicator::Communicator;
use murmur_slice::mumble_server::{Meta, MetaPrx};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Ice-контекст, подмешиваемый в каждый исходящий вызов.
///
/// Сюда попадает `secret`, заданный один раз на билдере. Раньше его надо было
/// вручную собирать в `HashMap` и передавать последним аргументом в **каждый**
/// вызов.
pub(crate) type Context = Arc<HashMap<String, String>>;

pub(crate) struct Shared {
    pub(crate) context: Context,
    pub(crate) request_timeout: Duration,
    // Настройки входящей стороны валидируются на connect(), а используются слоем
    // колбеков, которого пока нет.
    #[allow(dead_code)]
    pub(crate) callback_listen: std::net::SocketAddr,
    #[allow(dead_code)]
    pub(crate) callback_advertise: Option<(String, u16)>,
    #[allow(dead_code)]
    pub(crate) callback_identity_prefix: String,
}

impl Shared {
    /// Клон контекста для очередного вызова.
    pub(crate) fn ctx(&self) -> Option<HashMap<String, String>> {
        if self.context.is_empty() {
            None
        } else {
            Some((*self.context).clone())
        }
    }
}

struct ClientInner {
    shared: Arc<Shared>,
    /// Реестр подписок и владелец Ice-адаптера для входящих вызовов.
    registry: Arc<crate::events::Registry>,
    /// Прокси без мьютекса: методы сгенерированного кода теперь берут `&self`,
    /// а соединение мультиплексирует запросы по request_id.
    meta: MetaPrx,
    endpoint: Endpoint,
    /// Кэш разрешённых виртуальных серверов, чтобы `server()` не ходил на сервер
    /// повторно.
    servers: Mutex<BTreeMap<i32, VirtualServer>>,
}

/// Подключение к Murmur.
///
/// `Clone` дешёвый (внутри `Arc`), все методы берут `&self` — хендл можно
/// свободно раздавать по таскам без `Arc<Mutex<_>>` в пользовательском коде.
#[derive(Clone)]
pub struct MurmurClient {
    inner: Arc<ClientInner>,
}

impl MurmurClient {
    /// Частый случай: `MurmurClient::connect("127.0.0.1:6502")`.
    pub async fn connect<E: IntoEndpoint>(endpoint: E) -> Result<MurmurClient> {
        MurmurClient::builder().endpoint(endpoint)?.connect().await
    }

    pub fn builder() -> MurmurClientBuilder {
        MurmurClientBuilder::default()
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.inner.endpoint
    }

    pub(crate) fn shared(&self) -> &Arc<Shared> {
        &self.inner.shared
    }

    pub(crate) fn registry(&self) -> &Arc<crate::events::Registry> {
        &self.inner.registry
    }

    /// Собирает прокси по строке — нужно слою колбеков, чтобы отдать Murmur'у
    /// ссылку на наш servant.
    pub(crate) async fn make_proxy(&self, proxy_string: &str) -> Result<ice_rs::proxy::Proxy> {
        let mut comm = Communicator::new()
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        comm.string_to_proxy(proxy_string)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))
    }

    pub(crate) async fn add_meta_callback(
        &self,
        prx: &murmur_slice::mumble_server::MetaCallbackPrx,
    ) -> Result<()> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        meta.add_callback(prx, ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))
    }

    /// Подписаться на события уровня `Meta`: запуск и остановка виртуальных
    /// серверов.
    ///
    /// Внутренний `MetaCallback` фасад ставит сам при первой подписке на события
    /// сервера — он нужен для переподписки после перезапуска. Этот метод только
    /// добавляет ваш обработчик.
    pub async fn on_meta_events(
        &self,
        handler: Arc<dyn crate::events::MetaEvents>,
    ) -> Result<()> {
        self.inner.registry.add_meta_handler(handler).await;
        Ok(())
    }

    /// Снять все подписки и погасить адаптер входящих вызовов.
    pub async fn shutdown(&self) {
        self.inner.registry.shutdown().await;
    }

    // ── виртуальные серверы ───────────────────────────────────────────────

    /// Хендл виртуального сервера. Результат кэшируется.
    pub async fn server(&self, id: ServerId) -> Result<VirtualServer> {
        if let Some(s) = self.inner.servers.lock().await.get(&id.get()) {
            return Ok(s.clone());
        }
        let cx = FaultContext::new().server(id);
        let prx = {
            let meta = &self.inner.meta;
            let ctx = self.inner.shared.ctx();
            meta.get_server(id.get(), ctx)
                .await
                .map_err(|e| from_wire(e, cx))?
        };
        let vs = VirtualServer::new(id, prx, self.clone());
        self.inner
            .servers
            .lock()
            .await
            .insert(id.get(), vs.clone());
        Ok(vs)
    }

    /// Только запущенные виртуальные серверы.
    pub async fn booted_servers(&self) -> Result<Vec<VirtualServer>> {
        let list = {
            let meta = &self.inner.meta;
            let ctx = self.inner.shared.ctx();
            meta.get_booted_servers(ctx)
                .await
                .map_err(|e| from_wire(e, FaultContext::new()))?
        };
        self.wrap_server_list(list).await
    }

    /// Все определённые виртуальные серверы, включая остановленные.
    pub async fn servers(&self) -> Result<Vec<VirtualServer>> {
        let list = {
            let meta = &self.inner.meta;
            let ctx = self.inner.shared.ctx();
            meta.get_all_servers(ctx)
                .await
                .map_err(|e| from_wire(e, FaultContext::new()))?
        };
        self.wrap_server_list(list).await
    }

    /// Сахар для подавляющего большинства ботов: единственный запущенный сервер.
    pub async fn only_server(&self) -> Result<VirtualServer> {
        let mut booted = self.booted_servers().await?;
        booted.pop().ok_or(Error::NoBootedServer)
    }

    pub async fn create_server(&self) -> Result<VirtualServer> {
        let prx = {
            let meta = &self.inner.meta;
            let ctx = self.inner.shared.ctx();
            meta.new_server(ctx)
                .await
                .map_err(|e| from_wire(e, FaultContext::new()))?
        };
        let vs = VirtualServer::new(ServerId(-1), prx, self.clone());
        let id = vs.refresh_id().await?;
        self.inner.servers.lock().await.insert(id.get(), vs.clone());
        Ok(vs)
    }

    async fn wrap_server_list(
        &self,
        list: Vec<murmur_slice::mumble_server::ServerPrx>,
    ) -> Result<Vec<VirtualServer>> {
        let mut out = Vec::with_capacity(list.len());
        let mut cache = Vec::new();
        for prx in list {
            let vs = VirtualServer::new(ServerId(-1), prx, self.clone());
            let id = vs.refresh_id().await?;
            cache.push((id.get(), vs.clone()));
            out.push(vs);
        }
        let mut guard = self.inner.servers.lock().await;
        for (id, vs) in cache {
            guard.insert(id, vs);
        }
        Ok(out)
    }

    // ── информация ────────────────────────────────────────────────────────

    /// Заменяет четыре `&mut` out-параметра.
    pub async fn version(&self) -> Result<Version> {
        let mut major = 0i32;
        let mut minor = 0i32;
        let mut patch = 0i32;
        let mut text = String::new();
        {
            let meta = &self.inner.meta;
            let ctx = self.inner.shared.ctx();
            meta.get_version(&mut major, &mut minor, &mut patch, &mut text, ctx)
                .await
                .map_err(|e| from_wire(e, FaultContext::new()))?;
        }
        Ok(Version {
            major,
            minor,
            patch,
            text,
        })
    }

    pub async fn uptime(&self) -> Result<Duration> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        let secs = meta
            .get_uptime(ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        Ok(Duration::from_secs(secs.max(0) as u64))
    }

    pub async fn default_config(&self) -> Result<BTreeMap<String, String>> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        let m = meta
            .get_default_conf(ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        Ok(m.into_iter().collect())
    }

    /// Текст `MumbleServer.ice`, как его отдаёт сервер.
    pub async fn slice_source(&self) -> Result<String> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        meta.get_slice(ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))
    }

    pub async fn slice_checksums(&self) -> Result<BTreeMap<String, String>> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        let m = meta
            .get_slice_checksums(ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        Ok(m.into_iter().collect())
    }

    pub async fn database_state(&self) -> Result<DbState> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        let s = meta
            .get_assumed_database_state(ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        Ok(DbState::from(s))
    }

    pub async fn set_database_state(&self, state: DbState) -> Result<()> {
        let meta = &self.inner.meta;
        let ctx = self.inner.shared.ctx();
        let raw = murmur_slice::mumble_server::Dbstate::from(state);
        meta.set_assumed_database_state(&raw, ctx)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))
    }

    /// Escape hatch: сгенерированный прокси `Meta` с готовым контекстом.
    ///
    /// Обёрнуто около двух третей операций; остальное — здесь.
    pub async fn raw_meta(&self) -> crate::raw::RawMeta {
        crate::raw::RawMeta::new(self.inner.meta.clone(), self.inner.shared.clone())
    }
}

impl std::fmt::Debug for MurmurClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MurmurClient")
            .field("endpoint", &self.inner.endpoint.proxy_string())
            .field("has_secret", &!self.inner.shared.context.is_empty())
            .finish()
    }
}

/// Построитель подключения.
pub struct MurmurClientBuilder {
    endpoint: Option<Endpoint>,
    secret: Option<String>,
    extra_context: HashMap<String, String>,
    request_timeout: Duration,
    tls: Option<TlsConfig>,
    callback_listen: std::net::SocketAddr,
    callback_advertise: Option<(String, u16)>,
    callback_identity_prefix: String,
}

impl Default for MurmurClientBuilder {
    fn default() -> Self {
        MurmurClientBuilder {
            endpoint: None,
            secret: None,
            extra_context: HashMap::new(),
            request_timeout: Duration::from_secs(30),
            tls: None,
            callback_listen: "127.0.0.1:0".parse().expect("literal addr"),
            callback_advertise: None,
            callback_identity_prefix: String::from("mumble-ice"),
        }
    }
}

impl MurmurClientBuilder {
    pub fn endpoint<E: IntoEndpoint>(mut self, e: E) -> Result<Self> {
        self.endpoint = Some(e.into_endpoint()?);
        Ok(self)
    }

    pub fn host_port(mut self, host: impl Into<String>, port: u16) -> Self {
        let ident = self
            .endpoint
            .as_ref()
            .map(|e| e.identity.clone())
            .unwrap_or_else(|| String::from(Endpoint::DEFAULT_IDENTITY));
        self.endpoint = Some(Endpoint::new(host, port).identity(ident));
        self
    }

    /// Ice-идентичность объекта Meta. По умолчанию `Meta`.
    pub fn meta_identity(mut self, ident: impl Into<String>) -> Self {
        let e = self
            .endpoint
            .take()
            .unwrap_or_else(|| Endpoint::new("127.0.0.1", Endpoint::DEFAULT_PORT));
        self.endpoint = Some(e.identity(ident));
        self
    }

    /// `icesecretread`/`icesecretwrite` из `murmur.ini`.
    ///
    /// Задаётся **один раз** и дальше подмешивается в каждый вызов автоматически.
    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        let s = secret.into();
        self.secret = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Дополнительные записи Ice-контекста. Нужны редко.
    pub fn context_entry(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.extra_context.insert(k.into(), v.into());
        self
    }

    pub fn request_timeout(mut self, d: Duration) -> Self {
        self.request_timeout = d;
        self
    }

    pub fn tls(mut self, cfg: TlsConfig) -> Self {
        self.tls = Some(cfg);
        self
    }

    /// Где мы слушаем входящие вызовы Murmur'а (колбеки, аутентификатор).
    ///
    /// `0.0.0.0:0` — валидно, но тогда обязателен [`callback_advertise`].
    ///
    /// [`callback_advertise`]: Self::callback_advertise
    pub fn callback_listen(mut self, addr: std::net::SocketAddr) -> Self {
        self.callback_listen = addr;
        self
    }

    /// Адрес, который объявляется Murmur'у для обратных вызовов.
    ///
    /// Murmur звонит **наружу** на этот адрес, поэтому под Docker/NAT он не равен
    /// адресу прослушивания.
    pub fn callback_advertise(mut self, host: impl Into<String>, port: u16) -> Self {
        self.callback_advertise = Some((host.into(), port));
        self
    }

    /// Префикс Ice-идентичностей, которые мы отдаём Murmur'у. По умолчанию
    /// `mumble-ice`. Менять, если в одном процессе живёт несколько ботов.
    pub fn callback_identity_prefix(mut self, p: impl Into<String>) -> Self {
        self.callback_identity_prefix = p.into();
        self
    }

    pub async fn connect(self) -> Result<MurmurClient> {
        let endpoint = self
            .endpoint
            .ok_or_else(|| Error::config("не задан адрес: вызовите endpoint() или host_port()"))?;

        // Проверяем конфигурацию колбеков ЗДЕСЬ, а не при первой подписке: иначе
        // плохой адрес всплывёт секундами позже как невнятный InvalidCallback от
        // Murmur'а.
        if self.callback_advertise.is_none() && self.callback_listen.ip().is_unspecified() {
            return Err(Error::config(format!(
                "слушаем на wildcard-адресе {}, но не задан callback_advertise: \
                 Murmur должен знать, куда звонить обратно — укажите \
                 .callback_advertise(host, port)",
                self.callback_listen
            )));
        }

        if let Some(tls) = &self.tls {
            if endpoint.secure {
                apply_tls(tls)?;
            } else {
                return Err(Error::config(
                    "задан tls(), но адрес не ssl:// — уточните схему эндпоинта",
                ));
            }
        }

        let mut context = self.extra_context;
        if let Some(secret) = self.secret {
            context.insert(String::from("secret"), secret);
        }

        let mut comm = Communicator::new()
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        let proxy = comm
            .string_to_proxy(&endpoint.proxy_string())
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;
        let meta = MetaPrx::unchecked_cast(proxy)
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;

        // Нижний слой открывает соединения лениво — это нужно, чтобы прокси
        // можно было десериализовать без дозвона. Но `connect()`, который не
        // проверяет достижимость сервера, — неожиданное поведение: ошибка тогда
        // всплыла бы на первом же вызове где-то в глубине бота. Поэтому
        // соединение здесь устанавливаем принудительно.
        meta.proxy
            .connection()
            .await
            .map_err(|e| from_wire(e, FaultContext::new()))?;

        let shared = Arc::new(Shared {
            context: Arc::new(context),
            request_timeout: self.request_timeout,
            callback_listen: self.callback_listen,
            callback_advertise: self.callback_advertise,
            callback_identity_prefix: self.callback_identity_prefix,
        });

        Ok(MurmurClient {
            inner: Arc::new(ClientInner {
                registry: Arc::new(crate::events::Registry::new(shared.clone())),
                shared,
                meta,
                endpoint,
                servers: Mutex::new(BTreeMap::new()),
            }),
        })
    }
}

/// Нижний слой читает настройки TLS из глобальных Ice-свойств, поэтому здесь мы
/// их туда и раскладываем.
fn apply_tls(cfg: &TlsConfig) -> Result<()> {
    use ice_rs::communicator::INITDATA;

    let mut guard = INITDATA
        .lock()
        .map_err(|_| Error::config("глобальные Ice-свойства отравлены паникой"))?;
    let props = guard.properties_as_mut();
    if let Some(ca) = &cfg.ca_file {
        props.set("IceSSL.CAs", &ca.to_string_lossy());
    }
    if let Some(cert) = &cfg.cert_file {
        props.set("IceSSL.CertFile", &cert.to_string_lossy());
    }
    if let Some(key) = &cfg.key_file {
        props.set("IceSSL.KeyFile", &key.to_string_lossy());
    }
    if let Some(pw) = &cfg.key_password {
        props.set("IceSSL.Password", pw);
    }
    props.set(
        "IceSSL.VerifyPeer",
        if cfg.accept_invalid_certs { "0" } else { "2" },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wildcard без advertise — это ошибка конфигурации, и она должна всплыть на
    /// connect(), а не позже как InvalidCallback от Murmur'а.
    #[tokio::test]
    async fn wildcard_listen_without_advertise_is_rejected() {
        let err = MurmurClient::builder()
            .host_port("127.0.0.1", 6502)
            .callback_listen("0.0.0.0:0".parse().unwrap())
            .connect()
            .await
            .expect_err("должно упасть до попытки соединения");
        let msg = err.to_string();
        assert!(msg.contains("callback_advertise"), "{}", msg);
    }

    #[tokio::test]
    async fn wildcard_listen_with_advertise_passes_validation() {
        // Соединиться не сможем (порт закрыт), но проверка конфигурации должна
        // пройти — то есть ошибка будет транспортной, а не Config.
        let err = MurmurClient::builder()
            .host_port("127.0.0.1", 1)
            .callback_listen("0.0.0.0:0".parse().unwrap())
            .callback_advertise("bot.internal", 7100)
            .connect()
            .await
            .expect_err("порт 1 закрыт");
        assert!(
            !matches!(err, Error::Config(_)),
            "ожидали транспортную ошибку, получили {:?}",
            err
        );
    }

    #[tokio::test]
    async fn missing_endpoint_is_a_config_error() {
        let err = MurmurClient::builder().connect().await.unwrap_err();
        assert!(matches!(err, Error::Config(_)), "{:?}", err);
    }

    #[tokio::test]
    async fn tls_without_ssl_scheme_is_rejected() {
        let err = MurmurClient::builder()
            .host_port("127.0.0.1", 6502)
            .tls(TlsConfig::new())
            .connect()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Config(_)), "{:?}", err);
    }

    #[test]
    fn secret_lands_in_the_context_exactly_once() {
        let b = MurmurClientBuilder::default().secret("s3cret");
        assert_eq!(Some(String::from("s3cret")), b.secret);
        // Пустой секрет — это отсутствие секрета, а не пустая запись контекста.
        let b = MurmurClientBuilder::default().secret("");
        assert_eq!(None, b.secret);
    }

    #[test]
    fn shared_context_is_none_when_empty() {
        let s = Shared {
            context: Arc::new(HashMap::new()),
            request_timeout: Duration::from_secs(1),
            callback_listen: "127.0.0.1:0".parse().unwrap(),
            callback_advertise: None,
            callback_identity_prefix: String::from("x"),
        };
        assert!(s.ctx().is_none(), "пустой контекст не надо отправлять");

        let mut m = HashMap::new();
        m.insert(String::from("secret"), String::from("v"));
        let s = Shared {
            context: Arc::new(m),
            ..s
        };
        assert_eq!(1, s.ctx().unwrap().len());
    }

    #[test]
    fn host_port_preserves_custom_identity() {
        let b = MurmurClientBuilder::default()
            .meta_identity("Custom")
            .host_port("h", 1);
        assert_eq!("Custom", b.endpoint.unwrap().identity);
    }
}
