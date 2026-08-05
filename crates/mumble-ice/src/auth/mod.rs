//! Аутентификатор: Murmur спрашивает у нас, кому можно войти.
//!
//! Протокол Murmur'а построен на сентинельных числах — и именно это здесь
//! спрятано. Ни одно значение вида `-1`/`-2`/`-3` наружу не выходит.
//!
//! # Разница между «отказать» и «не моё»
//!
//! Самая важная вещь во всём модуле. `AuthResult::Denied` означает «пароль
//! неверный», а `AuthResult::FallThrough` — «этого имени я не знаю, спроси свою
//! базу». Перепутать их значит **заблокировать всех пользователей из базы
//! Murmur'а**, потому что для них ваш аутификатор ответит «неверный пароль»
//! вместо «не моё».
//!
//! # Murmur ждёт ответа
//!
//! Вызовы синхронные: виртуальный сервер стоит, пока мы не ответим. Поэтому
//! обработчики обязаны быть быстрыми, и поэтому в трейт **не передаётся**
//! `VirtualServer`: обратный вызов в `Server`/`Meta` отсюда вешает Murmur
//! (`MumbleServer.ice`, комментарий к `ServerAuthenticator`). Здесь это
//! запрещено типами, а не документацией.

mod shim;

use crate::error::Error;
use crate::ids::UserId;
use crate::model::UserInfo;
use async_trait::async_trait;
use std::collections::BTreeMap;

pub(crate) use shim::make_authenticator_subscription;

/// DER-кодированный сертификат.
#[derive(Clone, PartialEq, Eq)]
pub struct CertificateDer(pub Vec<u8>);

impl std::fmt::Debug for CertificateDer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CertificateDer({} байт)", self.0.len())
    }
}

/// Запрос на аутентификацию.
#[derive(Clone)]
pub struct AuthRequest {
    /// Имя, как его прислал клиент. Murmur сравнивает имена без учёта регистра —
    /// стоит делать так же, см. [`name_ci`].
    ///
    /// [`name_ci`]: Self::name_ci
    pub name: String,
    /// Может быть пустым: вход только по сертификату.
    pub password: String,
    /// Цепочка сертификатов клиента, лист первым.
    pub certificates: Vec<CertificateDer>,
    /// Внутренний хэш сертификата Murmur'а — то, что лежит в
    /// [`UserField::Hash`](crate::model::UserField::Hash).
    pub cert_hash: String,
    /// Истина только если цепочка проверена доверенным CA.
    ///
    /// Slice прямо предупреждает: данным внутри `certificates` нельзя верить,
    /// пока это ложь.
    pub cert_strong: bool,
}

impl AuthRequest {
    /// Имя в нижнем регистре — так же, как сравнивает Murmur.
    pub fn name_ci(&self) -> String {
        self.name.to_lowercase()
    }

    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }

    /// Сертификаты — только если цепочка действительно проверена.
    ///
    /// Делает предупреждение Slice неигнорируемым: без `cert_strong` доступа к
    /// содержимому нет.
    pub fn trusted_certificates(&self) -> Option<&[CertificateDer]> {
        if self.cert_strong {
            Some(&self.certificates)
        } else {
            None
        }
    }

    /// Хэш сертификата — только при проверенной цепочке.
    pub fn trusted_cert_hash(&self) -> Option<&str> {
        if self.cert_strong {
            Some(&self.cert_hash)
        } else {
            None
        }
    }
}

/// Пароль редактируется: этот тип рано или поздно попадёт в лог.
impl std::fmt::Debug for AuthRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthRequest")
            .field("name", &self.name)
            .field("password", &"<redacted>")
            .field("certificates", &self.certificates.len())
            .field("cert_hash", &self.cert_hash)
            .field("cert_strong", &self.cert_strong)
            .finish()
    }
}

/// Успешная аутентификация.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthOk {
    pub user_id: UserId,
    /// Переименовать пользователя на время сессии. `None` — оставить как есть.
    pub rename: Option<String>,
    /// Группы корневого канала на время этой сессии.
    pub groups: Vec<String>,
}

impl AuthOk {
    pub fn new(user_id: UserId) -> AuthOk {
        AuthOk {
            user_id,
            rename: None,
            groups: Vec::new(),
        }
    }

    pub fn rename(mut self, name: impl Into<String>) -> AuthOk {
        self.rename = Some(name.into());
        self
    }

    pub fn group(mut self, g: impl Into<String>) -> AuthOk {
        self.groups.push(g.into());
        self
    }

    pub fn groups<I, S>(mut self, gs: I) -> AuthOk
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.groups.extend(gs.into_iter().map(Into::into));
        self
    }
}

/// Итог аутентификации.
///
/// Скрывает сентинелы `authenticate`: id, `-1`, `-2`, `-3`.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthResult {
    /// Пользователь опознан.
    Ok(AuthOk),
    /// Неверные учётные данные.
    ///
    /// **Не** для случая «я не знаю такого имени» — для него `FallThrough`.
    /// Иначе все пользователи из базы Murmur'а перестанут входить.
    Denied,
    /// «Не моё»: Murmur проверит свою базу. Правильный ответ на любое незнакомое
    /// имя, и дефолт для всего трейта.
    FallThrough,
    /// Бэкенд недоступен. Murmur скажет клиенту повторить, а не «неверный
    /// пароль».
    Unavailable,
}

impl From<UserId> for AuthResult {
    fn from(id: UserId) -> AuthResult {
        AuthResult::Ok(AuthOk::new(id))
    }
}

impl From<AuthOk> for AuthResult {
    fn from(ok: AuthOk) -> AuthResult {
        AuthResult::Ok(ok)
    }
}

/// Результат поиска: «знаю» либо «не моё».
///
/// Один тип на все четыре схемы поиска Murmur'а: `getInfo` (bool + out),
/// `nameToId` (`-2`), `idToName` (пустая строка), `idToTexture` (пустая
/// текстура). Во всех случаях «не знаю» означает «Murmur посмотрит свою базу».
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup<T> {
    Found(T),
    Unknown,
}

impl<T> Lookup<T> {
    pub fn found(v: T) -> Lookup<T> {
        Lookup::Found(v)
    }

    /// Намеренно не `From<Option<T>>`: неявное превращение `None` в `Unknown` —
    /// это ровно тот способ, которым проваливаются в fall-through случайно.
    pub fn from_option(o: Option<T>) -> Lookup<T> {
        match o {
            Some(v) => Lookup::Found(v),
            None => Lookup::Unknown,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Lookup<U> {
        match self {
            Lookup::Found(v) => Lookup::Found(f(v)),
            Lookup::Unknown => Lookup::Unknown,
        }
    }

    pub fn is_found(&self) -> bool {
        matches!(self, Lookup::Found(_))
    }
}

/// Итог изменения: `1` / `0` / `-1` у Murmur'а.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    Ok,
    Failed,
    FallThrough,
}

/// Итог регистрации: id / `-1` / `-2`.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterResult {
    Ok(UserId),
    Failed,
    FallThrough,
}

/// Аутентификатор Murmur.
///
/// Покрывает **оба** Slice-интерфейса — `ServerAuthenticator` и
/// `ServerUpdatingAuthenticator`. У всех методов дефолт «не моё», поэтому
/// минимальный рабочий аутентификатор — одна реализация [`authenticate`].
///
/// [`authenticate`]: Self::authenticate
#[async_trait]
pub trait Authenticator: Send + Sync + 'static {
    // ── ServerAuthenticator ───────────────────────────────────────────────

    /// Опознать пользователя.
    ///
    /// Незнакомое имя — это [`AuthResult::FallThrough`], а не `Denied`.
    async fn authenticate(&self, _req: AuthRequest) -> AuthResult {
        AuthResult::FallThrough
    }

    /// Сведения о зарегистрированном пользователе.
    async fn user_info(&self, _id: UserId) -> Lookup<UserInfo> {
        Lookup::Unknown
    }

    /// Имя → id.
    async fn name_to_id(&self, _name: &str) -> Lookup<UserId> {
        Lookup::Unknown
    }

    /// id → имя.
    async fn id_to_name(&self, _id: UserId) -> Lookup<String> {
        Lookup::Unknown
    }

    /// id → аватар.
    async fn id_to_texture(&self, _id: UserId) -> Lookup<Vec<u8>> {
        Lookup::Unknown
    }

    // ── ServerUpdatingAuthenticator ───────────────────────────────────────

    async fn register_user(&self, _info: &UserInfo) -> RegisterResult {
        RegisterResult::FallThrough
    }

    async fn unregister_user(&self, _id: UserId) -> UpdateResult {
        UpdateResult::FallThrough
    }

    /// Список зарегистрированных, отфильтрованный по подстроке имени.
    ///
    /// `Lookup`, а не голая карта: в Slice здесь нет fall-through, поэтому
    /// аутентификатор, вернувший пустую карту, **скрывает всех пользователей
    /// базы** от `Server::getRegisteredUsers`. `Unknown` означает «не моё» и
    /// уезжает как пустая карта, но осознанно.
    async fn registered_users(&self, _filter: &str) -> Lookup<BTreeMap<UserId, String>> {
        Lookup::Unknown
    }

    async fn set_user_info(&self, _id: UserId, _info: &UserInfo) -> UpdateResult {
        UpdateResult::FallThrough
    }

    async fn set_texture(&self, _id: UserId, _texture: &[u8]) -> UpdateResult {
        UpdateResult::FallThrough
    }

    // ── диагностика ───────────────────────────────────────────────────────

    /// Паника или внутренняя ошибка в методах выше приходит сюда.
    ///
    /// На провод при этом уходит безопасное значение: fall-through для поисков и
    /// `Unavailable` для `authenticate`. Сломанный аутентификатор деградирует до
    /// «Murmur смотрит свою базу», а не до «никто не может войти».
    async fn on_error(&self, err: Error) {
        eprintln!("mumble-ice: ошибка аутентификатора: {}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_request_redacts_password() {
        let req = AuthRequest {
            name: String::from("alice"),
            password: String::from("hunter2"),
            certificates: vec![CertificateDer(vec![1, 2, 3])],
            cert_hash: String::from("abc"),
            cert_strong: false,
        };
        let s = format!("{:?}", req);
        assert!(!s.contains("hunter2"), "пароль в Debug: {}", s);
        assert!(s.contains("<redacted>"), "{}", s);
        assert!(s.contains("alice"));
    }

    /// Предупреждение Slice должно быть неигнорируемым: без проверенной цепочки
    /// содержимого сертификатов не видно.
    #[test]
    fn untrusted_certificates_are_not_reachable() {
        let mut req = AuthRequest {
            name: String::from("a"),
            password: String::new(),
            certificates: vec![CertificateDer(vec![1])],
            cert_hash: String::from("h"),
            cert_strong: false,
        };
        assert!(req.trusted_certificates().is_none());
        assert!(req.trusted_cert_hash().is_none());

        req.cert_strong = true;
        assert_eq!(1, req.trusted_certificates().unwrap().len());
        assert_eq!(Some("h"), req.trusted_cert_hash());
    }

    #[test]
    fn name_ci_matches_murmur_semantics() {
        let req = AuthRequest {
            name: String::from("AlIcE"),
            password: String::new(),
            certificates: vec![],
            cert_hash: String::new(),
            cert_strong: false,
        };
        assert_eq!("alice", req.name_ci());
    }

    #[test]
    fn auth_ok_builder() {
        let ok = AuthOk::new(UserId(7))
            .rename("Alice [staff]")
            .group("admin")
            .groups(vec!["mods", "vip"]);
        assert_eq!(UserId(7), ok.user_id);
        assert_eq!(Some(String::from("Alice [staff]")), ok.rename);
        assert_eq!(vec!["admin", "mods", "vip"], ok.groups);

        assert_eq!(AuthResult::Ok(AuthOk::new(UserId(1))), UserId(1).into());
    }

    #[test]
    fn lookup_conversions_are_explicit() {
        assert_eq!(Lookup::Found(5), Lookup::from_option(Some(5)));
        assert_eq!(Lookup::<i32>::Unknown, Lookup::from_option(None));
        assert!(Lookup::Found(1).is_found());
        assert_eq!(Lookup::Found(2), Lookup::Found(1).map(|v| v + 1));
        assert_eq!(Lookup::<i32>::Unknown, Lookup::<i32>::Unknown.map(|v| v + 1));
    }

    /// Сертификат в Debug не должен печататься целиком.
    #[test]
    fn certificate_debug_is_short() {
        let c = CertificateDer(vec![0xAB; 1000]);
        let s = format!("{:?}", c);
        assert!(s.contains("1000"), "{}", s);
        assert!(s.len() < 40, "слишком длинный Debug: {}", s);
    }
}
