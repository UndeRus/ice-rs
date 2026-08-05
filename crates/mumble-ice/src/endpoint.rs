//! Куда подключаться.
//!
//! Раньше пользователь писал Ice-прокси-строку руками
//! (`"Meta:tcp -h 127.0.0.1 -p 6502"`). Здесь принимается и она, и человеческие
//! формы вроде `"127.0.0.1:6502"` или `"ssl://murmur.example.com:6502"`.

use crate::error::{Error, Result};

/// Адрес Ice-эндпоинта Murmur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
    pub secure: bool,
    /// Ice-идентичность объекта Meta. Меняется только если её переименовали в
    /// `murmur.ini`.
    pub identity: String,
}

impl Endpoint {
    pub const DEFAULT_IDENTITY: &'static str = "Meta";
    pub const DEFAULT_PORT: u16 = 6502;

    pub fn new(host: impl Into<String>, port: u16) -> Endpoint {
        Endpoint {
            host: host.into(),
            port,
            secure: false,
            identity: String::from(Self::DEFAULT_IDENTITY),
        }
    }

    pub fn secure(mut self, yes: bool) -> Endpoint {
        self.secure = yes;
        self
    }

    pub fn identity(mut self, ident: impl Into<String>) -> Endpoint {
        self.identity = ident.into();
        self
    }

    /// Ice-прокси-строка для нижнего слоя.
    pub fn proxy_string(&self) -> String {
        format!(
            "{}:{} -h {} -p {}",
            self.identity,
            if self.secure { "ssl" } else { "tcp" },
            self.host,
            self.port
        )
    }

    /// Разбирает человеческую запись адреса.
    ///
    /// Понимает:
    /// - `host:port`, `host` (порт по умолчанию 6502)
    /// - `tcp://host:port`, `ssl://host:port`
    /// - готовую Ice-прокси-строку `Ident:tcp -h host -p port`
    pub fn parse(s: &str) -> Result<Endpoint> {
        let s = s.trim();
        if s.is_empty() {
            return Err(Error::config("пустой адрес"));
        }

        // Готовая Ice-прокси-строка: содержит `-h`/`-p` после двоеточия.
        if s.contains(" -h ") || s.contains(" -p ") {
            return Self::parse_ice_proxy_string(s);
        }

        let (secure, rest) = if let Some(r) = s.strip_prefix("ssl://") {
            (true, r)
        } else if let Some(r) = s.strip_prefix("tcp://") {
            (false, r)
        } else {
            (false, s)
        };

        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => {
                let port: u16 = p.parse().map_err(|_| {
                    Error::config(format!("не могу разобрать порт в адресе {:?}", s))
                })?;
                (h, port)
            }
            None => (rest, Self::DEFAULT_PORT),
        };

        if host.is_empty() {
            return Err(Error::config(format!("в адресе {:?} нет хоста", s)));
        }
        if port == 0 {
            return Err(Error::config(format!(
                "порт 0 нельзя использовать для подключения ({:?})",
                s
            )));
        }

        Ok(Endpoint {
            host: String::from(host),
            port,
            secure,
            identity: String::from(Self::DEFAULT_IDENTITY),
        })
    }

    fn parse_ice_proxy_string(s: &str) -> Result<Endpoint> {
        let (ident, tail) = s
            .split_once(':')
            .ok_or_else(|| Error::config(format!("не Ice-прокси-строка: {:?}", s)))?;
        let ident = ident.trim();
        if ident.is_empty() {
            return Err(Error::config(format!("нет идентичности в {:?}", s)));
        }

        let mut secure = false;
        let mut host: Option<String> = None;
        let mut port: Option<u16> = None;

        let mut it = tail.split_whitespace().peekable();
        if let Some(&proto) = it.peek() {
            match proto {
                "ssl" => {
                    secure = true;
                    it.next();
                }
                "tcp" | "default" => {
                    it.next();
                }
                _ => {}
            }
        }
        while let Some(tok) = it.next() {
            match tok {
                "-h" => host = it.next().map(String::from),
                "-p" => {
                    port = match it.next() {
                        Some(p) => Some(p.parse().map_err(|_| {
                            Error::config(format!("не могу разобрать порт в {:?}", s))
                        })?),
                        None => None,
                    }
                }
                // Прочие опции эндпоинта (`-t`, `-z`, …) нижний слой не
                // поддерживает; молча игнорировать их опаснее, чем сказать.
                other if other.starts_with('-') => {
                    return Err(Error::config(format!(
                        "опция эндпоинта {:?} не поддерживается",
                        other
                    )))
                }
                _ => {}
            }
        }

        Ok(Endpoint {
            host: host.ok_or_else(|| Error::config(format!("нет -h в {:?}", s)))?,
            port: port.unwrap_or(Self::DEFAULT_PORT),
            secure,
            identity: String::from(ident),
        })
    }
}

impl std::str::FromStr for Endpoint {
    type Err = Error;
    fn from_str(s: &str) -> Result<Endpoint> {
        Endpoint::parse(s)
    }
}

/// Всё, что можно принять как адрес подключения.
///
/// Свой трейт, а не `TryInto<Endpoint>`: у `core` есть blanket-реализация
/// `TryFrom<T> for T` с `Error = Infallible`, поэтому `TryInto` с нашим типом
/// ошибки не даёт передать уже готовый `Endpoint`.
pub trait IntoEndpoint {
    fn into_endpoint(self) -> Result<Endpoint>;
}

impl IntoEndpoint for Endpoint {
    fn into_endpoint(self) -> Result<Endpoint> {
        Ok(self)
    }
}

impl IntoEndpoint for &str {
    fn into_endpoint(self) -> Result<Endpoint> {
        Endpoint::parse(self)
    }
}

impl IntoEndpoint for String {
    fn into_endpoint(self) -> Result<Endpoint> {
        Endpoint::parse(&self)
    }
}

impl IntoEndpoint for &String {
    fn into_endpoint(self) -> Result<Endpoint> {
        Endpoint::parse(self)
    }
}

impl IntoEndpoint for std::net::SocketAddr {
    fn into_endpoint(self) -> Result<Endpoint> {
        Ok(Endpoint::new(self.ip().to_string(), self.port()))
    }
}

/// Настройки TLS для соединения с Murmur.
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    pub ca_file: Option<std::path::PathBuf>,
    pub cert_file: Option<std::path::PathBuf>,
    pub key_file: Option<std::path::PathBuf>,
    pub key_password: Option<String>,
    pub accept_invalid_certs: bool,
}

impl TlsConfig {
    pub fn new() -> TlsConfig {
        TlsConfig::default()
    }
    pub fn ca_file(mut self, p: impl Into<std::path::PathBuf>) -> Self {
        self.ca_file = Some(p.into());
        self
    }
    pub fn client_identity(
        mut self,
        cert: impl Into<std::path::PathBuf>,
        key: impl Into<std::path::PathBuf>,
    ) -> Self {
        self.cert_file = Some(cert.into());
        self.key_file = Some(key.into());
        self
    }
    pub fn key_password(mut self, pw: impl Into<String>) -> Self {
        self.key_password = Some(pw.into());
        self
    }
    /// Только для тестов.
    pub fn danger_accept_invalid_certs(mut self, yes: bool) -> Self {
        self.accept_invalid_certs = yes;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_port() {
        let e = Endpoint::parse("127.0.0.1:6502").unwrap();
        assert_eq!("127.0.0.1", e.host);
        assert_eq!(6502, e.port);
        assert!(!e.secure);
        assert_eq!("Meta", e.identity);
        assert_eq!("Meta:tcp -h 127.0.0.1 -p 6502", e.proxy_string());
    }

    #[test]
    fn bare_host_uses_default_port() {
        let e = Endpoint::parse("murmur.example.com").unwrap();
        assert_eq!("murmur.example.com", e.host);
        assert_eq!(6502, e.port);
    }

    #[test]
    fn parses_scheme_prefixes() {
        assert!(Endpoint::parse("ssl://host:6502").unwrap().secure);
        assert!(!Endpoint::parse("tcp://host:6502").unwrap().secure);
        assert_eq!(
            "Meta:ssl -h host -p 6502",
            Endpoint::parse("ssl://host:6502").unwrap().proxy_string()
        );
    }

    /// Готовая Ice-строка тоже должна приниматься — её пишут в murmur.ini.
    #[test]
    fn parses_ice_proxy_string() {
        let e = Endpoint::parse("Meta:tcp -h 127.0.0.1 -p 6502").unwrap();
        assert_eq!("Meta", e.identity);
        assert_eq!("127.0.0.1", e.host);
        assert_eq!(6502, e.port);

        let e = Endpoint::parse("Murmur/Meta:ssl -h m.example.com -p 6503").unwrap();
        assert_eq!("Murmur/Meta", e.identity);
        assert!(e.secure);
        assert_eq!(6503, e.port);
    }

    #[test]
    fn round_trips_through_proxy_string() {
        for s in [
            "Meta:tcp -h 127.0.0.1 -p 6502",
            "Meta:ssl -h murmur.example.com -p 6503",
        ] {
            let e = Endpoint::parse(s).unwrap();
            assert_eq!(s, e.proxy_string());
            assert_eq!(e, Endpoint::parse(&e.proxy_string()).unwrap());
        }
    }

    #[test]
    fn rejects_nonsense() {
        assert!(Endpoint::parse("").is_err());
        assert!(Endpoint::parse("host:notaport").is_err());
        assert!(Endpoint::parse("host:0").is_err(), "порт 0 не для подключения");
        assert!(Endpoint::parse(":6502").is_err(), "нет хоста");
    }

    /// Неподдерживаемые опции лучше отвергнуть, чем молча проигнорировать: иначе
    /// пользователь думает, что задал таймаут, а его нет.
    #[test]
    fn rejects_unsupported_endpoint_options() {
        let e = Endpoint::parse("Meta:tcp -h h -p 1 -t 5000");
        assert!(e.is_err());
        assert!(e.unwrap_err().to_string().contains("-t"));
    }

    #[test]
    fn identity_override() {
        let e = Endpoint::new("h", 1).identity("Custom").secure(true);
        assert_eq!("Custom:ssl -h h -p 1", e.proxy_string());
    }
}
