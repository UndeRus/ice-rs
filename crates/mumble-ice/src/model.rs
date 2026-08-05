//! Владеющие доменные типы.
//!
//! Отличия от сгенерированных: сентинелы спрятаны в `Option`, секунды стали
//! `Duration`, 16 байт адреса — `IpAddr`, пара `userid == -1 ? group` — enum, а
//! имена приведены к человеческому виду (`Acllist` → `Vec<Acl>`,
//! `Dbstate` → `DbState`).

use crate::ids::{ChannelId, SessionId, UserId};
use crate::perm::Permission;
use murmur_slice::mumble_server as slice;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;

/// Версия Murmur. Заменяет `getVersion(out major, out minor, out patch, out text)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub text: String,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.text.is_empty() {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(f, "{} ({}.{}.{})", self.text, self.major, self.minor, self.patch)
        }
    }
}

/// Версия клиента Mumble. Заменяет пару `version: int` / `version2: long`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ClientVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ClientVersion {
    /// Формат Mumble 1.5+: `major<<48 | minor<<32 | patch<<16`.
    pub fn from_v2(raw: i64) -> ClientVersion {
        ClientVersion {
            major: ((raw >> 48) & 0xFFFF) as u16,
            minor: ((raw >> 32) & 0xFFFF) as u16,
            patch: ((raw >> 16) & 0xFFFF) as u16,
        }
    }

    /// Старый 32-битный формат: `major<<16 | minor<<8 | patch`.
    pub fn from_legacy(raw: i32) -> ClientVersion {
        ClientVersion {
            major: ((raw >> 16) & 0xFFFF) as u16,
            minor: ((raw >> 8) & 0xFF) as u16,
            patch: (raw & 0xFF) as u16,
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.major == 0 && self.minor == 0 && self.patch == 0
    }
}

impl std::fmt::Display for ClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Состояние базы Murmur. В Slice называется `DBState`, в сгенерированном коде —
/// `Dbstate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbState {
    Normal,
    ReadOnly,
}

impl From<slice::Dbstate> for DbState {
    fn from(v: slice::Dbstate) -> DbState {
        match v {
            slice::Dbstate::Normal => DbState::Normal,
            slice::Dbstate::ReadOnly => DbState::ReadOnly,
        }
    }
}

impl From<DbState> for slice::Dbstate {
    fn from(v: DbState) -> slice::Dbstate {
        match v {
            DbState::Normal => slice::Dbstate::Normal,
            DbState::ReadOnly => slice::Dbstate::ReadOnly,
        }
    }
}

/// Murmur кладёт адрес как 16 байт (IPv4 отображён в IPv6).
pub(crate) fn decode_address(raw: &[u8]) -> Option<IpAddr> {
    if raw.len() != 16 {
        return None;
    }
    let mut octets = [0u8; 16];
    octets.copy_from_slice(raw);
    let v6 = Ipv6Addr::from(octets);
    // ::ffff:a.b.c.d — это IPv4.
    if let Some(v4) = v6.to_ipv4() {
        if octets[..10].iter().all(|b| *b == 0) && octets[10] == 0xFF && octets[11] == 0xFF {
            return Some(IpAddr::V4(v4));
        }
    }
    if v6.is_unspecified() {
        return None;
    }
    Some(IpAddr::V6(v6))
}

pub(crate) fn encode_address(addr: Option<IpAddr>) -> Vec<u8> {
    match addr {
        None => vec![0u8; 16],
        Some(IpAddr::V4(v4)) => {
            let mut out = vec![0u8; 10];
            out.extend([0xFF, 0xFF]);
            out.extend(v4.octets());
            out
        }
        Some(IpAddr::V6(v6)) => v6.octets().to_vec(),
    }
}

/// Подключённый пользователь.
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    /// Идентификатор соединения. Недействителен после переподключения.
    pub session: SessionId,
    /// `None` для незарегистрированного (анонимного). Сентинел `-1` наружу не
    /// протекает.
    pub id: Option<UserId>,
    pub name: String,
    pub channel: ChannelId,
    pub mute: bool,
    pub deaf: bool,
    pub suppress: bool,
    pub priority_speaker: bool,
    pub self_mute: bool,
    pub self_deaf: bool,
    pub recording: bool,
    pub online: Duration,
    pub idle: Duration,
    pub bytes_per_sec: i32,
    pub client: ClientVersion,
    pub release: String,
    pub os: String,
    pub os_version: String,
    pub identity: String,
    /// Base64, как присылает Murmur.
    pub plugin_context: String,
    pub comment: String,
    /// `None`, если Murmur не сообщил адрес.
    pub address: Option<IpAddr>,
    /// Идёт ли голос по UDP. В Slice это инвертированный `tcponly`.
    pub udp: bool,
    pub udp_ping: f32,
    pub tcp_ping: f32,
}

impl From<&slice::User> for User {
    fn from(u: &slice::User) -> User {
        let client = if u.version_2 != 0 {
            ClientVersion::from_v2(u.version_2)
        } else {
            ClientVersion::from_legacy(u.version)
        };
        User {
            session: SessionId(u.session),
            id: if u.userid < 0 { None } else { Some(UserId(u.userid)) },
            name: u.name.clone(),
            channel: ChannelId(u.channel),
            mute: u.mute,
            deaf: u.deaf,
            suppress: u.suppress,
            priority_speaker: u.priority_speaker,
            self_mute: u.self_mute,
            self_deaf: u.self_deaf,
            recording: u.recording,
            online: Duration::from_secs(u.onlinesecs.max(0) as u64),
            idle: Duration::from_secs(u.idlesecs.max(0) as u64),
            bytes_per_sec: u.bytespersec,
            client,
            release: u.release.clone(),
            os: u.os.clone(),
            os_version: u.osversion.clone(),
            identity: u.identity.clone(),
            plugin_context: u.context.clone(),
            comment: u.comment.clone(),
            address: decode_address(&u.address),
            udp: !u.tcponly,
            udp_ping: u.udp_ping,
            tcp_ping: u.tcp_ping,
        }
    }
}

impl From<slice::User> for User {
    fn from(u: slice::User) -> User {
        User::from(&u)
    }
}

impl User {
    /// Обратно в сгенерированный тип — для `setState`.
    pub fn to_slice(&self) -> slice::User {
        slice::User {
            session: self.session.get(),
            userid: self.id.map(|u| u.get()).unwrap_or(-1),
            mute: self.mute,
            deaf: self.deaf,
            suppress: self.suppress,
            priority_speaker: self.priority_speaker,
            self_mute: self.self_mute,
            self_deaf: self.self_deaf,
            recording: self.recording,
            channel: self.channel.get(),
            name: self.name.clone(),
            onlinesecs: self.online.as_secs() as i32,
            bytespersec: self.bytes_per_sec,
            version: 0,
            version_2: 0,
            release: self.release.clone(),
            os: self.os.clone(),
            osversion: self.os_version.clone(),
            identity: self.identity.clone(),
            context: self.plugin_context.clone(),
            comment: self.comment.clone(),
            address: encode_address(self.address),
            tcponly: !self.udp,
            idlesecs: self.idle.as_secs() as i32,
            udp_ping: self.udp_ping,
            tcp_ping: self.tcp_ping,
        }
    }

    pub fn is_registered(&self) -> bool {
        self.id.is_some()
    }
}

/// Канал.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    /// `None` для корневого канала.
    pub parent: Option<ChannelId>,
    pub links: Vec<ChannelId>,
    pub description: String,
    pub temporary: bool,
    pub position: i32,
}

impl From<&slice::Channel> for Channel {
    fn from(c: &slice::Channel) -> Channel {
        Channel {
            id: ChannelId(c.id),
            name: c.name.clone(),
            // У корня Murmur ставит parent == id == 0; отдельного сентинела нет.
            parent: if c.id == 0 { None } else { Some(ChannelId(c.parent)) },
            links: c.links.iter().map(|i| ChannelId(*i)).collect(),
            description: c.description.clone(),
            temporary: c.temporary,
            position: c.position,
        }
    }
}

impl From<slice::Channel> for Channel {
    fn from(c: slice::Channel) -> Channel {
        Channel::from(&c)
    }
}

impl Channel {
    pub fn to_slice(&self) -> slice::Channel {
        slice::Channel {
            id: self.id.get(),
            name: self.name.clone(),
            parent: self.parent.map(|p| p.get()).unwrap_or(0),
            links: self.links.iter().map(|c| c.get()).collect(),
            description: self.description.clone(),
            temporary: self.temporary,
            position: self.position,
        }
    }

    pub fn is_root(&self) -> bool {
        self.id == ChannelId::ROOT
    }
}

/// Дерево каналов из `getTree` — без `Box` и сокращённых имён.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTree {
    pub channel: Channel,
    pub users: Vec<User>,
    pub children: Vec<ChannelTree>,
}

impl From<&slice::Tree> for ChannelTree {
    fn from(t: &slice::Tree) -> ChannelTree {
        ChannelTree {
            channel: Channel::from(&t.c),
            users: t.users.iter().map(User::from).collect(),
            children: t
                .children
                .iter()
                .map(|c| ChannelTree::from(&**c))
                .collect(),
        }
    }
}

impl ChannelTree {
    /// Обход в глубину: `(глубина, узел)`.
    pub fn walk(&self) -> Vec<(usize, &ChannelTree)> {
        let mut out = Vec::new();
        self.walk_into(0, &mut out);
        out
    }

    fn walk_into<'a>(&'a self, depth: usize, out: &mut Vec<(usize, &'a ChannelTree)>) {
        out.push((depth, self));
        for c in &self.children {
            c.walk_into(depth + 1, out);
        }
    }

    pub fn find(&self, id: ChannelId) -> Option<&ChannelTree> {
        if self.channel.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }

    /// Поиск по пути имён от текущего узла, без учёта регистра.
    pub fn find_by_path(&self, path: &[&str]) -> Option<&ChannelTree> {
        match path.split_first() {
            None => Some(self),
            Some((head, rest)) => self
                .children
                .iter()
                .find(|c| c.channel.name.eq_ignore_ascii_case(head))
                .and_then(|c| c.find_by_path(rest)),
        }
    }

    pub fn all_users(&self) -> Vec<&User> {
        let mut out: Vec<&User> = self.users.iter().collect();
        for c in &self.children {
            out.extend(c.all_users());
        }
        out
    }
}

/// Кому адресована запись ACL.
///
/// В Slice это пара полей: `userid == -1` означает «смотри `group`».
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AclSubject {
    User(UserId),
    Group(String),
}

/// Запись ACL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acl {
    pub apply_here: bool,
    pub apply_subchannels: bool,
    /// Только для чтения: `setACL` унаследованные записи игнорирует.
    pub inherited: bool,
    pub subject: AclSubject,
    pub allow: Permission,
    pub deny: Permission,
}

impl From<&slice::Acl> for Acl {
    fn from(a: &slice::Acl) -> Acl {
        Acl {
            apply_here: a.apply_here,
            apply_subchannels: a.apply_subs,
            inherited: a.inherited,
            subject: if a.userid >= 0 {
                AclSubject::User(UserId(a.userid))
            } else {
                AclSubject::Group(a.group.clone())
            },
            allow: Permission::from_bits_truncate(a.allow),
            deny: Permission::from_bits_truncate(a.deny),
        }
    }
}

impl Acl {
    pub fn to_slice(&self) -> slice::Acl {
        let (userid, group) = match &self.subject {
            AclSubject::User(u) => (u.get(), String::new()),
            AclSubject::Group(g) => (-1, g.clone()),
        };
        slice::Acl {
            apply_here: self.apply_here,
            apply_subs: self.apply_subchannels,
            inherited: self.inherited,
            userid,
            group,
            allow: self.allow.bits(),
            deny: self.deny.bits(),
        }
    }
}

/// Группа на канале.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    /// Только для чтения.
    pub inherited: bool,
    pub inherit: bool,
    pub inheritable: bool,
    pub add: Vec<UserId>,
    pub remove: Vec<UserId>,
    /// Только для чтения: текущий состав, включая унаследованных.
    pub members: Vec<UserId>,
}

impl From<&slice::Group> for Group {
    fn from(g: &slice::Group) -> Group {
        Group {
            name: g.name.clone(),
            inherited: g.inherited,
            inherit: g.inherit,
            inheritable: g.inheritable,
            add: g.add.iter().map(|i| UserId(*i)).collect(),
            remove: g.remove.iter().map(|i| UserId(*i)).collect(),
            members: g.members.iter().map(|i| UserId(*i)).collect(),
        }
    }
}

impl Group {
    pub fn to_slice(&self) -> slice::Group {
        slice::Group {
            name: self.name.clone(),
            inherited: self.inherited,
            inherit: self.inherit,
            inheritable: self.inheritable,
            add: self.add.iter().map(|u| u.get()).collect(),
            remove: self.remove.iter().map(|u| u.get()).collect(),
            members: self.members.iter().map(|u| u.get()).collect(),
        }
    }
}

/// Снимок ACL канала. Заменяет три out-параметра `getACL`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AclSnapshot {
    pub acls: Vec<Acl>,
    pub groups: Vec<Group>,
    /// Наследует ли канал ACL от родителя.
    pub inherit_from_parent: bool,
}

impl AclSnapshot {
    /// Записи, определённые на самом канале, — то есть записываемый набор.
    pub fn own_acls(&self) -> impl Iterator<Item = &Acl> {
        self.acls.iter().filter(|a| !a.inherited)
    }

    pub fn own_groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.iter().filter(|g| !g.inherited)
    }

    pub fn group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.name == name)
    }

    /// Добавляет разрешающую запись.
    pub fn allow(&mut self, subject: AclSubject, perms: Permission) -> &mut Self {
        self.acls.push(Acl {
            apply_here: true,
            apply_subchannels: true,
            inherited: false,
            subject,
            allow: perms,
            deny: Permission::empty(),
        });
        self
    }

    /// Добавляет запрещающую запись.
    pub fn deny(&mut self, subject: AclSubject, perms: Permission) -> &mut Self {
        self.acls.push(Acl {
            apply_here: true,
            apply_subchannels: true,
            inherited: false,
            subject,
            allow: Permission::empty(),
            deny: perms,
        });
        self
    }
}

/// Бан.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ban {
    pub address: Option<IpAddr>,
    /// Длина префикса адреса.
    pub bits: i32,
    pub name: String,
    /// Хэш сертификата.
    pub cert_hash: String,
    pub reason: String,
    /// Unix-время начала.
    pub start: i64,
    /// `None` — бессрочно.
    pub duration: Option<Duration>,
}

impl From<&slice::Ban> for Ban {
    fn from(b: &slice::Ban) -> Ban {
        Ban {
            address: decode_address(&b.address),
            bits: b.bits,
            name: b.name.clone(),
            cert_hash: b.hash.clone(),
            reason: b.reason.clone(),
            start: b.start as i64,
            duration: if b.duration <= 0 {
                None
            } else {
                Some(Duration::from_secs(b.duration as u64))
            },
        }
    }
}

impl Ban {
    pub fn to_slice(&self) -> slice::Ban {
        slice::Ban {
            address: encode_address(self.address),
            bits: self.bits,
            name: self.name.clone(),
            hash: self.cert_hash.clone(),
            reason: self.reason.clone(),
            start: self.start as i32,
            duration: self.duration.map(|d| d.as_secs() as i32).unwrap_or(0),
        }
    }
}

/// Строка серверного лога.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Unix-время.
    pub timestamp: i64,
    pub text: String,
}

impl From<&slice::LogEntry> for LogEntry {
    fn from(e: &slice::LogEntry) -> LogEntry {
        LogEntry {
            timestamp: e.timestamp as i64,
            text: e.txt.clone(),
        }
    }
}

/// Поле информации о зарегистрированном пользователе.
///
/// В Slice enum называется `UserInfo`; имя освобождено под обёртку над картой.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserField {
    Name,
    Email,
    Comment,
    Hash,
    Password,
    LastActive,
    KdfIterations,
}

impl From<slice::UserInfo> for UserField {
    fn from(v: slice::UserInfo) -> UserField {
        match v {
            slice::UserInfo::UserName => UserField::Name,
            slice::UserInfo::UserEmail => UserField::Email,
            slice::UserInfo::UserComment => UserField::Comment,
            slice::UserInfo::UserHash => UserField::Hash,
            slice::UserInfo::UserPassword => UserField::Password,
            slice::UserInfo::UserLastActive => UserField::LastActive,
            slice::UserInfo::UserKDFIterations => UserField::KdfIterations,
        }
    }
}

impl From<UserField> for slice::UserInfo {
    fn from(v: UserField) -> slice::UserInfo {
        match v {
            UserField::Name => slice::UserInfo::UserName,
            UserField::Email => slice::UserInfo::UserEmail,
            UserField::Comment => slice::UserInfo::UserComment,
            UserField::Hash => slice::UserInfo::UserHash,
            UserField::Password => slice::UserInfo::UserPassword,
            UserField::LastActive => slice::UserInfo::UserLastActive,
            UserField::KdfIterations => slice::UserInfo::UserKDFIterations,
        }
    }
}

/// Информация о регистрации. Обёртка над `UserInfoMap`.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct UserInfo {
    fields: BTreeMap<UserField, String>,
}

/// Пароль редактируется: эта структура рано или поздно попадёт в лог.
impl std::fmt::Debug for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("UserInfo");
        for (k, v) in &self.fields {
            if *k == UserField::Password {
                d.field("Password", &"<redacted>");
            } else {
                d.field(&format!("{:?}", k), v);
            }
        }
        d.finish()
    }
}

impl UserInfo {
    /// Имя обязательно — так требует Slice.
    pub fn new(name: impl Into<String>) -> UserInfo {
        let mut fields = BTreeMap::new();
        fields.insert(UserField::Name, name.into());
        UserInfo { fields }
    }

    pub fn empty() -> UserInfo {
        UserInfo::default()
    }

    pub fn get(&self, f: UserField) -> Option<&str> {
        self.fields.get(&f).map(|s| s.as_str())
    }

    pub fn set(&mut self, f: UserField, v: impl Into<String>) -> &mut Self {
        self.fields.insert(f, v.into());
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.get(UserField::Name)
    }
    pub fn email(&self) -> Option<&str> {
        self.get(UserField::Email)
    }
    pub fn comment(&self) -> Option<&str> {
        self.get(UserField::Comment)
    }
    pub fn cert_hash(&self) -> Option<&str> {
        self.get(UserField::Hash)
    }
    pub fn kdf_iterations(&self) -> Option<u32> {
        self.get(UserField::KdfIterations).and_then(|s| s.parse().ok())
    }

    pub fn with_email(mut self, v: impl Into<String>) -> Self {
        self.set(UserField::Email, v);
        self
    }
    pub fn with_comment(mut self, v: impl Into<String>) -> Self {
        self.set(UserField::Comment, v);
        self
    }
    /// Открытым текстом; Murmur сам его хэширует.
    pub fn with_password(mut self, v: impl Into<String>) -> Self {
        self.set(UserField::Password, v);
        self
    }
    pub fn with_cert_hash(mut self, v: impl Into<String>) -> Self {
        self.set(UserField::Hash, v);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (UserField, &str)> {
        self.fields.iter().map(|(k, v)| (*k, v.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub(crate) fn to_slice(&self) -> slice::UserInfoMap {
        self.fields
            .iter()
            .map(|(k, v)| (slice::UserInfo::from(*k), v.clone()))
            .collect()
    }

    pub(crate) fn from_slice(m: &slice::UserInfoMap) -> UserInfo {
        UserInfo {
            fields: m
                .iter()
                .map(|(k, v)| (UserField::from(*k), v.clone()))
                .collect(),
        }
    }
}

/// Результат проверки пароля.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordCheck {
    Ok(UserId),
    WrongPassword,
    NoSuchUser,
}

/// Текстовое сообщение от пользователя.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMessage {
    pub sessions: Vec<SessionId>,
    pub channels: Vec<ChannelId>,
    pub trees: Vec<ChannelId>,
    pub text: String,
}

impl From<&slice::TextMessage> for TextMessage {
    fn from(m: &slice::TextMessage) -> TextMessage {
        TextMessage {
            sessions: m.sessions.iter().map(|s| SessionId(*s)).collect(),
            channels: m.channels.iter().map(|c| ChannelId(*c)).collect(),
            trees: m.trees.iter().map(|c| ChannelId(*c)).collect(),
            text: m.text.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn slice_user() -> slice::User {
        slice::User {
            session: 5,
            userid: -1,
            mute: false,
            deaf: false,
            suppress: false,
            priority_speaker: false,
            self_mute: true,
            self_deaf: false,
            recording: false,
            channel: 3,
            name: String::from("alice"),
            onlinesecs: 120,
            bytespersec: 1000,
            version: 0,
            version_2: (1i64 << 48) | (5i64 << 32) | (735i64 << 16),
            release: String::from("1.5.735"),
            os: String::from("Linux"),
            osversion: String::from("6.1"),
            identity: String::new(),
            context: String::new(),
            comment: String::new(),
            address: {
                let mut a = vec![0u8; 10];
                a.extend([0xFF, 0xFF, 192, 168, 1, 7]);
                a
            },
            tcponly: false,
            idlesecs: 7,
            udp_ping: 1.5,
            tcp_ping: 2.5,
        }
    }

    /// Сентинел `-1` для анонимного пользователя не должен протекать наружу.
    #[test]
    fn anonymous_user_id_becomes_none() {
        let u = User::from(&slice_user());
        assert_eq!(None, u.id);
        assert!(!u.is_registered());

        let mut raw = slice_user();
        raw.userid = 42;
        assert_eq!(Some(UserId(42)), User::from(&raw).id);
    }

    #[test]
    fn seconds_become_durations_and_tcponly_is_inverted() {
        let u = User::from(&slice_user());
        assert_eq!(Duration::from_secs(120), u.online);
        assert_eq!(Duration::from_secs(7), u.idle);
        assert!(u.udp, "tcponly=false должно означать udp=true");
    }

    #[test]
    fn address_decodes_ipv4_mapped() {
        let u = User::from(&slice_user());
        assert_eq!(
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 7))),
            u.address
        );
    }

    #[test]
    fn address_round_trips() {
        for addr in [
            Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            Some(IpAddr::V6("2001:db8::1".parse().unwrap())),
            None,
        ] {
            let raw = encode_address(addr);
            assert_eq!(16, raw.len());
            assert_eq!(addr, decode_address(&raw), "не сошлось для {:?}", addr);
        }
    }

    #[test]
    fn client_version_v2_and_legacy() {
        let u = User::from(&slice_user());
        assert_eq!(
            ClientVersion { major: 1, minor: 5, patch: 735 },
            u.client
        );
        assert_eq!("1.5.735", u.client.to_string());

        let mut raw = slice_user();
        raw.version_2 = 0;
        raw.version = (1 << 16) | (3 << 8) | 4;
        assert_eq!(
            ClientVersion { major: 1, minor: 3, patch: 4 },
            User::from(&raw).client
        );
    }

    #[test]
    fn user_round_trips_through_slice() {
        let u = User::from(&slice_user());
        let back = User::from(&u.to_slice());
        // version/version_2 не отправляются обратно (Murmur их игнорирует в
        // setState), поэтому сравниваем всё остальное.
        assert_eq!(u.session, back.session);
        assert_eq!(u.id, back.id);
        assert_eq!(u.name, back.name);
        assert_eq!(u.channel, back.channel);
        assert_eq!(u.online, back.online);
        assert_eq!(u.idle, back.idle);
        assert_eq!(u.udp, back.udp);
        assert_eq!(u.address, back.address);
        assert_eq!(u.self_mute, back.self_mute);
    }

    #[test]
    fn root_channel_has_no_parent() {
        let root = slice::Channel {
            id: 0,
            name: String::from("Root"),
            parent: 0,
            links: vec![],
            description: String::new(),
            temporary: false,
            position: 0,
        };
        let c = Channel::from(&root);
        assert!(c.is_root());
        assert_eq!(None, c.parent);

        let child = slice::Channel { id: 3, parent: 0, ..root };
        assert_eq!(Some(ChannelId::ROOT), Channel::from(&child).parent);
    }

    /// Пара `userid == -1 ? group` схлопывается в enum.
    #[test]
    fn acl_subject_collapses_the_sentinel_pair() {
        let by_user = slice::Acl {
            apply_here: true,
            apply_subs: false,
            inherited: false,
            userid: 7,
            group: String::new(),
            allow: 0x0C,
            deny: 0,
        };
        let a = Acl::from(&by_user);
        assert_eq!(AclSubject::User(UserId(7)), a.subject);
        assert!(a.allow.contains(Permission::ENTER | Permission::SPEAK));

        let by_group = slice::Acl {
            userid: -1,
            group: String::from("admin"),
            ..by_user
        };
        assert_eq!(
            AclSubject::Group(String::from("admin")),
            Acl::from(&by_group).subject
        );
    }

    #[test]
    fn acl_round_trips() {
        for subject in [
            AclSubject::User(UserId(3)),
            AclSubject::Group(String::from("mods")),
        ] {
            let a = Acl {
                apply_here: true,
                apply_subchannels: true,
                inherited: false,
                subject: subject.clone(),
                allow: Permission::SPEAK,
                deny: Permission::KICK,
            };
            assert_eq!(a, Acl::from(&a.to_slice()));
        }
    }

    #[test]
    fn acl_snapshot_filters_inherited() {
        let mk = |inherited| Acl {
            apply_here: true,
            apply_subchannels: false,
            inherited,
            subject: AclSubject::Group(String::from("g")),
            allow: Permission::empty(),
            deny: Permission::empty(),
        };
        let snap = AclSnapshot {
            acls: vec![mk(true), mk(false)],
            groups: vec![],
            inherit_from_parent: true,
        };
        assert_eq!(1, snap.own_acls().count());
    }

    #[test]
    fn permanent_ban_has_no_duration() {
        let raw = slice::Ban {
            address: vec![0u8; 16],
            bits: 128,
            name: String::from("bad"),
            hash: String::new(),
            reason: String::from("spam"),
            start: 1000,
            duration: 0,
        };
        assert_eq!(None, Ban::from(&raw).duration);
        let timed = slice::Ban { duration: 60, ..raw };
        assert_eq!(Some(Duration::from_secs(60)), Ban::from(&timed).duration);
    }

    /// Пароль не должен утекать в лог.
    #[test]
    fn user_info_debug_redacts_password() {
        let info = UserInfo::new("alice").with_password("hunter2").with_email("a@b.c");
        let s = format!("{:?}", info);
        assert!(!s.contains("hunter2"), "пароль в Debug: {}", s);
        assert!(s.contains("<redacted>"), "{}", s);
        assert!(s.contains("alice"));
    }

    #[test]
    fn user_info_round_trips() {
        let info = UserInfo::new("bob").with_email("b@example.com").with_comment("hi");
        let back = UserInfo::from_slice(&info.to_slice());
        assert_eq!(info, back);
        assert_eq!(Some("bob"), back.name());
        assert_eq!(Some("b@example.com"), back.email());
    }

    #[test]
    fn channel_tree_navigation() {
        let leaf = slice::Tree {
            c: slice::Channel {
                id: 2,
                name: String::from("Sub"),
                parent: 0,
                links: vec![],
                description: String::new(),
                temporary: false,
                position: 0,
            },
            children: vec![],
            users: vec![slice_user()],
        };
        let root = slice::Tree {
            c: slice::Channel {
                id: 0,
                name: String::from("Root"),
                parent: 0,
                links: vec![],
                description: String::new(),
                temporary: false,
                position: 0,
            },
            children: vec![Box::new(leaf)],
            users: vec![],
        };
        let t = ChannelTree::from(&root);
        assert_eq!(2, t.walk().len());
        assert_eq!(vec![0, 1], t.walk().iter().map(|(d, _)| *d).collect::<Vec<_>>());
        assert_eq!("Sub", t.find(ChannelId(2)).unwrap().channel.name);
        assert!(t.find(ChannelId(99)).is_none());
        assert_eq!("Sub", t.find_by_path(&["sub"]).unwrap().channel.name);
        assert_eq!(1, t.all_users().len());
    }
}
