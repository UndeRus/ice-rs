use std::collections::HashMap;
use crate::encoding::*;

#[derive(Debug)]
pub enum MessageType {
    Reply(Header, RawReply),
    ValidateConnection(Header),
    /// Входящий запрос на исходящем соединении. Bidirectional-соединения не
    /// поддерживаются (Murmur звонит наружу на наш адаптер), поэтому кадр
    /// только фиксируется — но он больше не роняет reader.
    Request(Header),
    CloseConnection(Header),
}

/// Коды статуса ответа Ice.
pub mod reply_status {
    pub const OK: u8 = 0;
    pub const USER_EXCEPTION: u8 = 1;
    pub const OBJECT_NOT_EXIST: u8 = 2;
    pub const FACET_NOT_EXIST: u8 = 3;
    pub const OPERATION_NOT_EXIST: u8 = 4;
    pub const UNKNOWN_LOCAL_EXCEPTION: u8 = 5;
    pub const UNKNOWN_USER_EXCEPTION: u8 = 6;
    pub const UNKNOWN_EXCEPTION: u8 = 7;
}

/// Ответ, разобранный только до статуса: тело остаётся сырым.
///
/// Интерпретация статуса — забота вызывающего, а не reader-таска. Раньше
/// `ReplyData::from_bytes` возвращал `Err` для статусов 2..=7 прямо внутри
/// reader'а, тот умирал, ошибка терялась, и вызывающий вместо настоящей ошибки
/// получал таймаут 30 секунд.
#[derive(Debug)]
pub struct RawReply {
    pub request_id: i32,
    pub status: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct Header {
    pub magic: String,
    pub protocol_major: u8,
    pub protocol_minor: u8,
    pub encoding_major: u8,
    pub encoding_minor: u8,
    pub message_type: u8,
    pub compression_status: u8,
    pub message_size: i32
}

#[derive(Debug, IceDerive)]
pub struct Identity {
    pub name: String,
    pub category: String
}

impl Identity {
    pub fn new(ident: &str) -> Identity {
        match ident.find("/") {
            Some(_) => {
                let split = ident.split("/").collect::<Vec<&str>>();
                Identity {
                    name: String::from(split[1]),
                    category: String::from(split[0])
                }
            }
            None => Identity {
                name: String::from(ident),
                category: String::new()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Encapsulation {
    pub size: i32,
    pub major: u8,
    pub minor: u8,
    pub data: Vec<u8>
}

#[derive(Debug, IceDerive)]
pub struct RequestData {
    pub request_id: i32,
    pub id: Identity,
    pub facet: Vec<String>,
    pub operation: String,
    pub mode: u8,
    pub context: HashMap<String, String>,
    pub params: Encapsulation
}

#[derive(Debug, IceEncode)]
pub struct ReplyData {
    pub request_id: i32,
    pub status: u8,
    pub body: Encapsulation
}

#[derive(Debug, IceDerive)]
pub struct Version
{
    pub major: u8,
    pub minor: u8
}

/// Поля в порядке кодирования Ice: identity (name, category), facet, …
#[derive(Debug, IceDerive)]
pub struct ProxyData {
    pub name: String,
    pub category: String,
    pub facet: Vec<String>,
    pub mode: u8,
    pub secure: bool,
    pub protocol: Version,
    pub encoding: Version,
}

impl ProxyData {
    /// Строка идентичности для proxy-string (`category/name` либо только `name`).
    pub fn identity_string(&self) -> String {
        if self.category.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", self.category, self.name)
        }
    }
}

#[derive(Debug)]
pub enum EndPointType {
    WellKnownObject(String),
    TCP(EndpointData),
    SSL(EndpointData),
}

#[derive(Debug)]
pub struct LocatorResult {
    pub proxy_data: ProxyData,
    pub size: IceSize,
    pub endpoint: EndPointType
}

#[derive(Debug, IceDerive)]
pub struct EndpointData
{
    pub host: String,
    pub port: i32,
    pub timeout: i32,
    pub compress: bool
}

impl Header {
    pub fn new(message_type: u8, message_size: i32) -> Header {
        Header {
            magic: String::from("IceP"),
            protocol_major: 1,
            protocol_minor: 0,
            encoding_major: 1,
            encoding_minor: 0,
            message_type: message_type,
            compression_status: 0,
            message_size: message_size
        }
    }
}

impl Encapsulation {
    pub fn empty() -> Encapsulation {
        Encapsulation {
            size: 6,
            major: 1,
            minor: 1,
            data: vec![]
        }
    }

    pub fn from(bytes: Vec<u8>) -> Encapsulation {
        Encapsulation {
            size: 6 + bytes.len() as i32,
            major: 1,
            minor: 1,
            data: bytes
        }
    }
}

/// Маршалит ссылку на прокси в кодировке Ice 1.1 (`Reference::streamWrite`).
///
/// Порядок полей: identity, facet, mode, secure, protocol, encoding,
/// **количество эндпоинтов**, затем на каждый эндпоинт — `short` с типом
/// транспорта и Encapsulation с его параметрами.
///
/// Вынесено из `ToBytes for Proxy` отдельной функцией, чтобы это можно было
/// проверить фикстурой: у `Proxy` внутри живой сокет, и в юнит-тесте его не
/// собрать.
pub fn marshal_proxy_ref(
    ident: &str,
    host: &str,
    port: i32,
    secure: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    let id = Identity::new(ident);
    let proxy_data = ProxyData {
        name: id.name,
        category: id.category,
        facet: vec![],
        mode: 0, // Twoway
        secure,
        protocol: Version { major: 1, minor: 0 },
        encoding: Version { major: 1, minor: 1 },
    };
    let mut out = proxy_data.to_bytes()?;
    out.extend(IceSize { size: 1 }.to_bytes()?);
    let ep_type: i16 = if secure { 2 } else { 1 };
    out.extend(ep_type.to_bytes()?);
    let ep = EndpointData {
        host: String::from(host),
        port,
        timeout: -1,
        compress: false,
    };
    out.extend(Encapsulation::from(ep.to_bytes()?).to_bytes()?);
    Ok(out)
}

/// Убирает до двух внешних Encapsulation с параметров входящего запроса (совместимость с Ice C++).
pub fn peel_slice_param_payload(data: &[u8]) -> Vec<u8> {
    let mut cur = data.to_vec();
    for _ in 0..2 {
        let mut r = 0i32;
        if cur.len() < 6 {
            break;
        }
        match Encapsulation::from_bytes(&cur, &mut r) {
            Ok(enc) if (r as usize) == cur.len() => cur = enc.data,
            _ => break,
        }
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Эталон для `cb:tcp -h 127.0.0.1 -p 7100`, собранный вручную по спецификации
    /// кодирования прокси Ice 1.1. Именно этот тест ловит дефект, из-за которого
    /// Murmur не мог разобрать наш callback-прокси.
    fn expected_cb_proxy_bytes() -> Vec<u8> {
        let mut want: Vec<u8> = Vec::new();
        // identity.name = "cb"
        want.extend([0x02, b'c', b'b']);
        // identity.category = "" (пустая строка — только нулевой размер)
        want.push(0x00);
        // facet: пустая StringSeq
        want.push(0x00);
        // mode = 0 (Twoway)
        want.push(0x00);
        // secure = false
        want.push(0x00);
        // protocol = 1.0
        want.extend([0x01, 0x00]);
        // encoding = 1.1
        want.extend([0x01, 0x01]);
        // КОЛИЧЕСТВО эндпоинтов = 1 (а не длина их байтов!)
        want.push(0x01);
        // тип эндпоинта: short 1 = TCP, little-endian
        want.extend([0x01, 0x00]);
        // Encapsulation с параметрами эндпоинта:
        //   payload = host(10) + port(4) + timeout(4) + compress(1) = 19
        //   size = 6 + 19 = 25
        want.extend(25i32.to_le_bytes());
        want.extend([0x01, 0x01]); // encoding внутри инкапсуляции
        want.extend([0x09]); // размер "127.0.0.1"
        want.extend(b"127.0.0.1");
        want.extend(7100i32.to_le_bytes());
        want.extend((-1i32).to_le_bytes()); // timeout
        want.push(0x00); // compress = false
        want
    }

    #[test]
    fn proxy_ref_matches_ice_wire_format() {
        let got = marshal_proxy_ref("cb", "127.0.0.1", 7100, false).unwrap();
        assert_eq!(
            expected_cb_proxy_bytes(),
            got,
            "маршалинг прокси разошёлся с кодировкой Ice 1.1"
        );
    }

    /// Регрессия на конкретный дефект: в позиции счётчика эндпоинтов раньше
    /// оказывалась длина блоба (27 байт для этого адреса), и настоящий Ice читал
    /// её как «27 эндпоинтов».
    #[test]
    fn endpoint_count_is_a_count_not_a_byte_length() {
        let got = marshal_proxy_ref("cb", "127.0.0.1", 7100, false).unwrap();
        // 3 (name) + 1 (category) + 1 (facet) + 1 (mode) + 1 (secure) + 2 + 2 = 11
        let count_pos = 11;
        assert_eq!(
            1, got[count_pos],
            "в позиции счётчика эндпоинтов должно быть 1, а не длина блоба"
        );
        let endpoint_blob_len = got.len() - count_pos - 1;
        assert_ne!(
            endpoint_blob_len as u8, got[count_pos],
            "счётчик снова равен длине байтов эндпоинта — дефект вернулся"
        );
    }

    #[test]
    fn ssl_proxy_uses_endpoint_type_two_and_secure_flag() {
        let got = marshal_proxy_ref("cb", "127.0.0.1", 7100, true).unwrap();
        assert_eq!(0x01, got[6], "secure должен быть выставлен");
        assert_eq!(0x02, got[12], "тип эндпоинта ssl = 2");
    }

    /// `category/name` должен разъезжаться по двум полям identity.
    #[test]
    fn proxy_ref_splits_identity_category() {
        let got = marshal_proxy_ref("cat/name", "127.0.0.1", 7100, false).unwrap();
        let mut want: Vec<u8> = vec![0x04];
        want.extend(b"name");
        want.push(0x03);
        want.extend(b"cat");
        assert_eq!(&want[..], &got[..want.len()]);
    }
}