//! Виртуальный сервер — то, с чем бот работает большую часть времени.

use crate::client::{MurmurClient, Shared};
use crate::error::{from_wire, Error, FaultContext, Result};
use crate::ids::{ChannelId, ServerId, SessionId, UserId};
use crate::model::{
    Acl, AclSnapshot, AclSubject, Ban, Channel, ChannelTree, Group, LogEntry, PasswordCheck,
    User, UserInfo,
};
use crate::perm::Permission;
use murmur_slice::mumble_server::{self as slice, Server, ServerPrx};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Выполняет вызов на прокси: подмешивает контекст, ограничивает по времени и
/// разбирает ошибку в вариант [`Error`].
///
/// Таймаут нужен и здесь, а не только в нижнем слое: там он захардкожен, а тут
/// задаётся билдером.
macro_rules! call {
    ($self:expr, $cx:expr, |$prx:ident, $ctx:ident| $body:expr) => {{
        // Ни мьютекса, ни `&mut`: методы прокси берут `&self`, а соединение
        // мультиплексирует запросы по request_id — поэтому вызовы к одному
        // серверу идут параллельно.
        let $prx = &$self.inner.prx;
        let $ctx = $self.inner.shared.ctx();
        let deadline = $self.inner.shared.request_timeout;
        match tokio::time::timeout(deadline, $body).await {
            Ok(r) => r.map_err(|e| from_wire(e, $cx)),
            Err(_) => Err(Error::Timeout {
                op: "server request",
                after: deadline,
            }),
        }
    }};
}

struct ServerInner {
    id: AtomicI32,
    prx: ServerPrx,
    shared: Arc<Shared>,
    client: MurmurClient,
}

/// Хендл виртуального сервера.
///
/// `Clone` дешёвый, все методы берут `&self`.
#[derive(Clone)]
pub struct VirtualServer {
    inner: Arc<ServerInner>,
}

impl VirtualServer {
    pub(crate) fn new(id: ServerId, prx: ServerPrx, client: MurmurClient) -> VirtualServer {
        let shared = client.shared().clone();
        VirtualServer {
            inner: Arc::new(ServerInner {
                id: AtomicI32::new(id.get()),
                prx,
                shared,
                client,
            }),
        }
    }

    /// Известный id — без обращения к серверу.
    pub fn id(&self) -> ServerId {
        ServerId(self.inner.id.load(Ordering::Relaxed))
    }

    pub fn client(&self) -> &MurmurClient {
        &self.inner.client
    }

    /// Спрашивает id у сервера и запоминает. Нужно после `getAllServers`, где
    /// приходят только прокси.
    pub(crate) async fn refresh_id(&self) -> Result<ServerId> {
        let cx = FaultContext::new();
        let id = call!(self, cx, |prx, ctx| prx.id(ctx))?;
        self.inner.id.store(id, Ordering::Relaxed);
        Ok(ServerId(id))
    }

    fn cx(&self) -> FaultContext {
        FaultContext::new().server(self.id())
    }

    // ── жизненный цикл ────────────────────────────────────────────────────

    pub async fn is_running(&self) -> Result<bool> {
        let cx = self.cx();
        call!(self, cx, |prx, ctx| prx.is_running(ctx))
    }

    pub async fn start(&self) -> Result<()> {
        let cx = self.cx();
        call!(self, cx, |prx, ctx| prx.start(ctx))
    }

    pub async fn stop(&self) -> Result<()> {
        let cx = self.cx();
        call!(self, cx, |prx, ctx| prx.stop(ctx))
    }

    /// `Server::delete`. Имя нарочно длинное: `delete` рядом со `stop` в
    /// автодополнении — это инцидент в продакшене.
    pub async fn delete_permanently(self) -> Result<()> {
        let cx = self.cx();
        call!(self, cx, |prx, ctx| prx.delete(ctx))
    }

    pub async fn uptime(&self) -> Result<Duration> {
        let cx = self.cx();
        let secs = call!(self, cx, |prx, ctx| prx.get_uptime(ctx))?;
        Ok(Duration::from_secs(secs.max(0) as u64))
    }

    // ── пользователи ──────────────────────────────────────────────────────

    pub async fn users(&self) -> Result<Vec<User>> {
        let cx = self.cx();
        let m = call!(self, cx, |prx, ctx| prx.get_users(ctx))?;
        Ok(m.values().map(User::from).collect())
    }

    pub async fn users_by_session(&self) -> Result<HashMap<SessionId, User>> {
        let cx = self.cx();
        let m = call!(self, cx, |prx, ctx| prx.get_users(ctx))?;
        Ok(m.iter()
            .map(|(k, v)| (SessionId(*k), User::from(v)))
            .collect())
    }

    pub async fn user(&self, s: SessionId) -> Result<User> {
        let cx = self.cx().session(s);
        let u = call!(self, cx, |prx, ctx| prx.get_state(s.get(), ctx))?;
        Ok(User::from(&u))
    }

    /// `None` вместо ошибки — обычная проверка «а он ещё тут?».
    pub async fn try_user(&self, s: SessionId) -> Result<Option<User>> {
        match self.user(s).await {
            Ok(u) => Ok(Some(u)),
            Err(e) if e.is_stale_handle() => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Поиск по имени, без учёта регистра — Murmur тоже так делает.
    pub async fn find_user(&self, name: &str) -> Result<Option<User>> {
        let users = self.users().await?;
        Ok(users
            .into_iter()
            .find(|u| u.name.eq_ignore_ascii_case(name)))
    }

    pub async fn kick(&self, s: SessionId, reason: &str) -> Result<()> {
        let cx = self.cx().session(s);
        let reason = String::from(reason);
        call!(self, cx, |prx, ctx| prx.kick_user(s.get(), &reason, ctx))
    }

    pub async fn certificates(&self, s: SessionId) -> Result<Vec<Vec<u8>>> {
        let cx = self.cx().session(s);
        call!(self, cx, |prx, ctx| prx.get_certificate_list(s.get(), ctx))
    }

    /// Читает состояние, даёт его изменить и отправляет обратно одним `setState`.
    ///
    /// Единственный безопасный способ менять пользователя: `setState` отправляет
    /// структуру целиком, поэтому собирать её с нуля значит затирать поля,
    /// которые изменил кто-то другой.
    pub async fn update_user<F>(&self, s: SessionId, f: F) -> Result<()>
    where
        F: FnOnce(&mut User) + Send,
    {
        let mut u = self.user(s).await?;
        f(&mut u);
        self.set_user_state(&u).await
    }

    /// Отправляет состояние пользователя, которое у вас уже есть.
    pub async fn set_user_state(&self, u: &User) -> Result<()> {
        let cx = self.cx().session(u.session);
        let raw = u.to_slice();
        call!(self, cx, |prx, ctx| prx.set_state(&raw, ctx))
    }

    pub async fn mute(&self, s: SessionId, on: bool) -> Result<()> {
        self.update_user(s, |u| u.mute = on).await
    }

    pub async fn deafen(&self, s: SessionId, on: bool) -> Result<()> {
        self.update_user(s, |u| u.deaf = on).await
    }

    pub async fn suppress(&self, s: SessionId, on: bool) -> Result<()> {
        self.update_user(s, |u| u.suppress = on).await
    }

    pub async fn set_priority_speaker(&self, s: SessionId, on: bool) -> Result<()> {
        self.update_user(s, |u| u.priority_speaker = on).await
    }

    pub async fn move_user(&self, s: SessionId, to: ChannelId) -> Result<()> {
        self.update_user(s, |u| u.channel = to).await
    }

    pub async fn set_comment(&self, s: SessionId, comment: &str) -> Result<()> {
        let comment = String::from(comment);
        self.update_user(s, move |u| u.comment = comment).await
    }

    // ── каналы ────────────────────────────────────────────────────────────

    pub async fn channels(&self) -> Result<Vec<Channel>> {
        let cx = self.cx();
        let m = call!(self, cx, |prx, ctx| prx.get_channels(ctx))?;
        Ok(m.values().map(Channel::from).collect())
    }

    pub async fn channels_by_id(&self) -> Result<HashMap<ChannelId, Channel>> {
        let cx = self.cx();
        let m = call!(self, cx, |prx, ctx| prx.get_channels(ctx))?;
        Ok(m.iter()
            .map(|(k, v)| (ChannelId(*k), Channel::from(v)))
            .collect())
    }

    pub async fn channel(&self, c: ChannelId) -> Result<Channel> {
        let cx = self.cx().channel(c);
        let raw = call!(self, cx, |prx, ctx| prx.get_channel_state(c.get(), ctx))?;
        Ok(Channel::from(&raw))
    }

    pub async fn try_channel(&self, c: ChannelId) -> Result<Option<Channel>> {
        match self.channel(c).await {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.is_stale_handle() => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn find_channel_by_name(&self, name: &str) -> Result<Option<Channel>> {
        let all = self.channels().await?;
        Ok(all
            .into_iter()
            .find(|c| c.name.eq_ignore_ascii_case(name)))
    }

    /// Дерево каналов как настоящий рекурсивный тип.
    pub async fn tree(&self) -> Result<ChannelTree> {
        let cx = self.cx();
        let raw = call!(self, cx, |prx, ctx| prx.get_tree(ctx))?;
        Ok(ChannelTree::from(&raw))
    }

    pub async fn create_channel(&self, name: &str, parent: ChannelId) -> Result<ChannelId> {
        let cx = self.cx().channel(parent);
        let name = String::from(name);
        let id = call!(self, cx, |prx, ctx| prx.add_channel(&name, parent.get(), ctx))?;
        Ok(ChannelId(id))
    }

    pub async fn remove_channel(&self, c: ChannelId) -> Result<()> {
        let cx = self.cx().channel(c);
        call!(self, cx, |prx, ctx| prx.remove_channel(c.get(), ctx))
    }

    /// Читает состояние канала, даёт изменить и отправляет обратно.
    pub async fn update_channel<F>(&self, c: ChannelId, f: F) -> Result<()>
    where
        F: FnOnce(&mut Channel) + Send,
    {
        let mut ch = self.channel(c).await?;
        f(&mut ch);
        self.set_channel_state(&ch).await
    }

    pub async fn set_channel_state(&self, c: &Channel) -> Result<()> {
        let cx = self.cx().channel(c.id);
        let raw = c.to_slice();
        call!(self, cx, |prx, ctx| prx.set_channel_state(&raw, ctx))
    }

    pub async fn rename_channel(&self, c: ChannelId, name: &str) -> Result<()> {
        let name = String::from(name);
        self.update_channel(c, move |ch| ch.name = name).await
    }

    pub async fn set_channel_description(&self, c: ChannelId, d: &str) -> Result<()> {
        let d = String::from(d);
        self.update_channel(c, move |ch| ch.description = d).await
    }

    pub async fn move_channel(&self, c: ChannelId, new_parent: ChannelId) -> Result<()> {
        self.update_channel(c, move |ch| ch.parent = Some(new_parent))
            .await
    }

    pub async fn link_channels(&self, a: ChannelId, b: ChannelId) -> Result<()> {
        self.update_channel(a, move |ch| {
            if !ch.links.contains(&b) {
                ch.links.push(b);
            }
        })
        .await
    }

    pub async fn unlink_channels(&self, a: ChannelId, b: ChannelId) -> Result<()> {
        self.update_channel(a, move |ch| ch.links.retain(|l| *l != b))
            .await
    }

    // ── текстовые сообщения ───────────────────────────────────────────────

    pub async fn message_user(&self, s: SessionId, text: &str) -> Result<()> {
        let cx = self.cx().session(s);
        let text = String::from(text);
        call!(self, cx, |prx, ctx| prx.send_message(s.get(), &text, ctx))
    }

    pub async fn message_channel(&self, c: ChannelId, text: &str) -> Result<()> {
        let cx = self.cx().channel(c);
        let text = String::from(text);
        call!(self, cx, |prx, ctx| prx.send_message_channel(
            c.get(),
            false,
            &text,
            ctx
        ))
    }

    /// Канал и все подканалы.
    pub async fn message_channel_tree(&self, c: ChannelId, text: &str) -> Result<()> {
        let cx = self.cx().channel(c);
        let text = String::from(text);
        call!(self, cx, |prx, ctx| prx.send_message_channel(
            c.get(),
            true,
            &text,
            ctx
        ))
    }

    /// Всем на сервере.
    pub async fn broadcast(&self, text: &str) -> Result<()> {
        self.message_channel_tree(ChannelId::ROOT, text).await
    }

    pub async fn send_welcome_message(&self, to: &[UserId]) -> Result<()> {
        let cx = self.cx();
        let ids: Vec<i32> = to.iter().map(|u| u.get()).collect();
        call!(self, cx, |prx, ctx| prx.send_welcome_message(&ids, ctx))
    }

    // ── зарегистрированные пользователи ───────────────────────────────────

    pub async fn register(&self, info: UserInfo) -> Result<UserId> {
        let cx = self.cx();
        let raw = info.to_slice();
        let id = call!(self, cx, |prx, ctx| prx.register_user(&raw, ctx))?;
        Ok(UserId(id))
    }

    pub async fn register_named(&self, name: &str, password: Option<&str>) -> Result<UserId> {
        let mut info = UserInfo::new(name);
        if let Some(pw) = password {
            info = info.with_password(pw);
        }
        self.register(info).await
    }

    pub async fn unregister(&self, u: UserId) -> Result<()> {
        let cx = self.cx().user(u);
        call!(self, cx, |prx, ctx| prx.unregister_user(u.get(), ctx))
    }

    pub async fn registration(&self, u: UserId) -> Result<UserInfo> {
        let cx = self.cx().user(u);
        let raw = call!(self, cx, |prx, ctx| prx.get_registration(u.get(), ctx))?;
        Ok(UserInfo::from_slice(&raw))
    }

    pub async fn update_registration(&self, u: UserId, info: UserInfo) -> Result<()> {
        let cx = self.cx().user(u);
        let raw = info.to_slice();
        call!(self, cx, |prx, ctx| prx.update_registration(
            u.get(),
            &raw,
            ctx
        ))
    }

    pub async fn set_password(&self, u: UserId, pw: &str) -> Result<()> {
        let mut info = self.registration(u).await?;
        info.set(crate::model::UserField::Password, pw);
        self.update_registration(u, info).await
    }

    pub async fn set_superuser_password(&self, pw: &str) -> Result<()> {
        let cx = self.cx();
        let pw = String::from(pw);
        call!(self, cx, |prx, ctx| prx.set_superuser_password(&pw, ctx))
    }

    /// Пустой фильтр — все.
    pub async fn registered_users(&self, filter: &str) -> Result<BTreeMap<UserId, String>> {
        let cx = self.cx();
        let filter = String::from(filter);
        let m = call!(self, cx, |prx, ctx| prx.get_registered_users(&filter, ctx))?;
        Ok(m.into_iter().map(|(k, v)| (UserId(k), v)).collect())
    }

    /// Проверка пароля.
    ///
    /// Murmur различает «неверный пароль» (-1) и «нет такого пользователя» (-2);
    /// здесь оба дают `None`, потому что бот, который различает их наружу,
    /// становится оракулом для перебора имён. Нужна разница — есть
    /// [`verify_password_detailed`].
    ///
    /// [`verify_password_detailed`]: Self::verify_password_detailed
    pub async fn verify_password(&self, name: &str, pw: &str) -> Result<Option<UserId>> {
        Ok(match self.verify_password_detailed(name, pw).await? {
            PasswordCheck::Ok(id) => Some(id),
            _ => None,
        })
    }

    pub async fn verify_password_detailed(&self, name: &str, pw: &str) -> Result<PasswordCheck> {
        let cx = self.cx();
        let name = String::from(name);
        let pw = String::from(pw);
        let id = call!(self, cx, |prx, ctx| prx.verify_password(&name, &pw, ctx))?;
        Ok(match id {
            -1 => PasswordCheck::WrongPassword,
            -2 => PasswordCheck::NoSuchUser,
            id => PasswordCheck::Ok(UserId(id)),
        })
    }

    /// Аватар. Пустой ответ Murmur'а — это `None`.
    pub async fn texture(&self, u: UserId) -> Result<Option<Vec<u8>>> {
        let cx = self.cx().user(u);
        let tex = call!(self, cx, |prx, ctx| prx.get_texture(u.get(), ctx))?;
        Ok(if tex.is_empty() { None } else { Some(tex) })
    }

    pub async fn set_texture(&self, u: UserId, tex: &[u8]) -> Result<()> {
        let cx = self.cx().user(u);
        let tex = tex.to_vec();
        call!(self, cx, |prx, ctx| prx.set_texture(u.get(), &tex, ctx))
    }

    pub async fn clear_texture(&self, u: UserId) -> Result<()> {
        self.set_texture(u, &[]).await
    }

    // ── ACL и группы ──────────────────────────────────────────────────────

    /// Заменяет три out-параметра `getACL`.
    pub async fn acl(&self, c: ChannelId) -> Result<AclSnapshot> {
        let cx = self.cx().channel(c);
        let mut acls: slice::Acllist = Vec::new();
        let mut groups: slice::GroupList = Vec::new();
        let mut inherit = false;
        {
            let ctx = self.inner.shared.ctx();
            self.inner
                .prx
                .get_acl(c.get(), &mut acls, &mut groups, &mut inherit, ctx)
                .await
                .map_err(|e| from_wire(e, cx))?;
        }
        Ok(AclSnapshot {
            acls: acls.iter().map(Acl::from).collect(),
            groups: groups.iter().map(Group::from).collect(),
            inherit_from_parent: inherit,
        })
    }

    /// Отправляет ACL целиком. Унаследованные записи отбрасываются: `setACL` их
    /// всё равно игнорирует, а отправлять их обратно — верный способ запутаться.
    pub async fn set_acl(&self, c: ChannelId, snap: &AclSnapshot) -> Result<()> {
        let cx = self.cx().channel(c);
        let acls: slice::Acllist = snap.own_acls().map(|a| a.to_slice()).collect();
        let groups: slice::GroupList = snap.own_groups().map(|g| g.to_slice()).collect();
        let inherit = snap.inherit_from_parent;
        call!(self, cx, |prx, ctx| prx.set_acl(
            c.get(),
            &acls,
            &groups,
            inherit,
            ctx
        ))
    }

    /// Читает ACL, даёт изменить и отправляет обратно.
    ///
    /// Единственный безопасный способ править ACL: `setACL` заменяет весь набор.
    pub async fn update_acl<F>(&self, c: ChannelId, f: F) -> Result<()>
    where
        F: FnOnce(&mut AclSnapshot) + Send,
    {
        let mut snap = self.acl(c).await?;
        f(&mut snap);
        self.set_acl(c, &snap).await
    }

    pub async fn add_to_group(&self, c: ChannelId, s: SessionId, group: &str) -> Result<()> {
        let cx = self.cx().channel(c).session(s);
        let group = String::from(group);
        call!(self, cx, |prx, ctx| prx.add_user_to_group(
            c.get(),
            s.get(),
            &group,
            ctx
        ))
    }

    pub async fn remove_from_group(&self, c: ChannelId, s: SessionId, group: &str) -> Result<()> {
        let cx = self.cx().channel(c).session(s);
        let group = String::from(group);
        call!(self, cx, |prx, ctx| prx.remove_user_from_group(
            c.get(),
            s.get(),
            &group,
            ctx
        ))
    }

    pub async fn has_permission(
        &self,
        s: SessionId,
        c: ChannelId,
        perm: Permission,
    ) -> Result<bool> {
        let cx = self.cx().channel(c).session(s);
        let bits = perm.bits();
        call!(self, cx, |prx, ctx| prx.has_permission(
            s.get(),
            c.get(),
            bits,
            ctx
        ))
    }

    pub async fn effective_permissions(&self, s: SessionId, c: ChannelId) -> Result<Permission> {
        let cx = self.cx().channel(c).session(s);
        let bits = call!(self, cx, |prx, ctx| prx.effective_permissions(
            s.get(),
            c.get(),
            ctx
        ))?;
        // truncate, а не from_bits: Murmur может прислать бит из будущей версии,
        // и ронять на этом весь вызов незачем.
        Ok(Permission::from_bits_truncate(bits))
    }

    // ── баны ──────────────────────────────────────────────────────────────

    pub async fn bans(&self) -> Result<Vec<Ban>> {
        let cx = self.cx();
        let list = call!(self, cx, |prx, ctx| prx.get_bans(ctx))?;
        Ok(list.iter().map(Ban::from).collect())
    }

    /// Заменяет список банов целиком. Имя такое, чтобы разрушительность была
    /// очевидна.
    pub async fn replace_bans(&self, bans: &[Ban]) -> Result<()> {
        let cx = self.cx();
        let raw: slice::BanList = bans.iter().map(|b| b.to_slice()).collect();
        call!(self, cx, |prx, ctx| prx.set_bans(&raw, ctx))
    }

    /// Дописывает бан к существующему списку (в Slice такой операции нет).
    pub async fn add_ban(&self, ban: Ban) -> Result<()> {
        let mut bans = self.bans().await?;
        bans.push(ban);
        self.replace_bans(&bans).await
    }

    /// Удаляет баны по предикату, возвращает сколько удалил.
    pub async fn remove_bans<F>(&self, mut pred: F) -> Result<usize>
    where
        F: FnMut(&Ban) -> bool + Send,
    {
        let bans = self.bans().await?;
        let before = bans.len();
        let kept: Vec<Ban> = bans.into_iter().filter(|b| !pred(b)).collect();
        let removed = before - kept.len();
        if removed > 0 {
            self.replace_bans(&kept).await?;
        }
        Ok(removed)
    }

    // ── слушатели каналов (Mumble 1.4+) ───────────────────────────────────

    pub async fn start_listening(&self, u: UserId, c: ChannelId) -> Result<()> {
        let cx = self.cx().user(u).channel(c);
        call!(self, cx, |prx, ctx| prx.start_listening(
            u.get(),
            c.get(),
            ctx
        ))
    }

    pub async fn stop_listening(&self, u: UserId, c: ChannelId) -> Result<()> {
        let cx = self.cx().user(u).channel(c);
        call!(self, cx, |prx, ctx| prx.stop_listening(u.get(), c.get(), ctx))
    }

    pub async fn is_listening(&self, u: UserId, c: ChannelId) -> Result<bool> {
        let cx = self.cx().user(u).channel(c);
        call!(self, cx, |prx, ctx| prx.is_listening(u.get(), c.get(), ctx))
    }

    pub async fn listening_channels(&self, u: UserId) -> Result<Vec<ChannelId>> {
        let cx = self.cx().user(u);
        let l = call!(self, cx, |prx, ctx| prx.get_listening_channels(u.get(), ctx))?;
        Ok(l.into_iter().map(ChannelId).collect())
    }

    pub async fn listening_users(&self, c: ChannelId) -> Result<Vec<UserId>> {
        let cx = self.cx().channel(c);
        let l = call!(self, cx, |prx, ctx| prx.get_listening_users(c.get(), ctx))?;
        Ok(l.into_iter().map(UserId).collect())
    }

    /// Обратите внимание на порядок аргументов: в Slice он у этой пары операций
    /// обратный относительно `startListening`. Здесь он единообразный.
    pub async fn listener_volume(&self, u: UserId, c: ChannelId) -> Result<f32> {
        let cx = self.cx().user(u).channel(c);
        call!(self, cx, |prx, ctx| prx
            .get_listener_volume_adjustment(c.get(), u.get(), ctx))
    }

    pub async fn set_listener_volume(&self, u: UserId, c: ChannelId, volume: f32) -> Result<()> {
        let cx = self.cx().user(u).channel(c);
        call!(self, cx, |prx, ctx| prx.set_listener_volume_adjustment(
            c.get(),
            u.get(),
            volume,
            ctx
        ))
    }

    // ── конфигурация и лог ────────────────────────────────────────────────

    /// Пустая строка от Murmur'а — это `None`.
    pub async fn config(&self, key: &str) -> Result<Option<String>> {
        let cx = self.cx();
        let key = String::from(key);
        let v = call!(self, cx, |prx, ctx| prx.get_conf(&key, ctx))?;
        Ok(if v.is_empty() { None } else { Some(v) })
    }

    pub async fn all_config(&self) -> Result<BTreeMap<String, String>> {
        let cx = self.cx();
        let m = call!(self, cx, |prx, ctx| prx.get_all_conf(ctx))?;
        Ok(m.into_iter().collect())
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let cx = self.cx();
        let key = String::from(key);
        let value = String::from(value);
        call!(self, cx, |prx, ctx| prx.set_conf(&key, &value, ctx))
    }

    pub async fn log_len(&self) -> Result<u32> {
        let cx = self.cx();
        let n = call!(self, cx, |prx, ctx| prx.get_log_len(ctx))?;
        Ok(n.max(0) as u32)
    }

    /// Записи лога — полуинтервал `[first, end)`, индекс 0 — самая свежая.
    ///
    /// Границы именно полуоткрытые: проверено на живом Murmur 1.5.857 —
    /// при `getLogLen() == 142` вызов `getLog(0, 141)` отдаёт 141 запись, а не
    /// 142. Документация Slice на этот счёт не высказывается.
    pub async fn log(&self, first: u32, end: u32) -> Result<Vec<LogEntry>> {
        if end <= first {
            return Ok(Vec::new());
        }
        let cx = self.cx();
        let (f, l) = (first as i32, end as i32);
        let list = call!(self, cx, |prx, ctx| prx.get_log(f, l, ctx))?;
        Ok(list.iter().map(LogEntry::from).collect())
    }

    /// Весь лог на момент вызова.
    ///
    /// Лог пополняется, в том числе от наших же вызовов, поэтому результат — это
    /// снимок, а не величина, согласованная с отдельно взятым [`log_len`].
    ///
    /// [`log_len`]: Self::log_len
    pub async fn all_log(&self) -> Result<Vec<LogEntry>> {
        let len = self.log_len().await?;
        self.log(0, len).await
    }

    pub async fn update_certificate(
        &self,
        cert_pem: &str,
        key_pem: &str,
        passphrase: Option<&str>,
    ) -> Result<()> {
        let cx = self.cx();
        let cert = String::from(cert_pem);
        let key = String::from(key_pem);
        let pass = String::from(passphrase.unwrap_or(""));
        call!(self, cx, |prx, ctx| prx.update_certificate(
            &cert, &key, &pass, ctx
        ))
    }

    // ── колбеки ───────────────────────────────────────────────────────────

    /// Подписаться на события сервера.
    ///
    /// Адаптер для входящих вызовов поднимается на первой подписке; адрес при
    /// этом уже проверен на `connect()`.
    ///
    /// ```no_run
    /// # use mumble_ice::prelude::*;
    /// # use std::sync::Arc;
    /// struct Greeter;
    /// #[async_trait::async_trait]
    /// impl ServerEvents for Greeter {
    ///     async fn user_connected(&self, srv: &VirtualServer, u: User) -> mumble_ice::Result<()> {
    ///         srv.message_user(u.session, &format!("привет, {}!", u.name)).await
    ///     }
    /// }
    /// # async fn f(srv: VirtualServer) -> mumble_ice::Result<()> {
    /// let sub = srv.on_events(Arc::new(Greeter)).await?;
    /// sub.forget();
    /// # Ok(()) }
    /// ```
    pub async fn on_events(
        &self,
        handler: Arc<dyn crate::events::ServerEvents>,
    ) -> Result<crate::events::Subscription> {
        crate::events::make_subscription(
            self.inner.client.registry(),
            self,
            crate::events::Registration::ServerEvents(handler),
        )
        .await
    }

    /// Те же события, но как поток — для ботов, чей главный цикл уже
    /// `select!`-ится.
    pub async fn events(&self) -> Result<crate::events::EventStream> {
        self.events_with(256, crate::events::Overflow::DropNewest).await
    }

    /// Поток событий с явной ёмкостью и политикой переполнения.
    pub async fn events_with(
        &self,
        capacity: usize,
        policy: crate::events::Overflow,
    ) -> Result<crate::events::EventStream> {
        let (bridge, rx) = crate::events::StreamBridge::new(capacity, policy);
        let sub = self.on_events(bridge.clone()).await?;
        Ok(bridge.into_stream(rx, sub))
    }

    /// Добавить контекстное действие в меню Mumble у указанного пользователя.
    pub async fn add_context_action(
        &self,
        session: SessionId,
        action: crate::events::ContextAction,
        handler: Arc<dyn crate::events::ContextHandler>,
    ) -> Result<crate::events::Subscription> {
        crate::events::make_subscription(
            self.inner.client.registry(),
            self,
            crate::events::Registration::Context {
                session,
                action,
                handler,
            },
        )
        .await
    }

    // ── низкоуровневые помощники для слоя колбеков ─────────────────────────

    pub(crate) async fn add_server_callback(&self, proxy_string: &str) -> Result<()> {
        let cx = self.cx();
        let prx = self.callback_proxy(proxy_string).await?;
        call!(self, cx, |prx_srv, ctx| prx_srv.add_callback(&prx, ctx))
    }

    pub(crate) async fn remove_server_callback(&self, proxy_string: &str) -> Result<()> {
        let cx = self.cx();
        let prx = self.callback_proxy(proxy_string).await?;
        call!(self, cx, |prx_srv, ctx| prx_srv.remove_callback(&prx, ctx))
    }

    pub(crate) async fn add_context_callback(
        &self,
        session: SessionId,
        action: &crate::events::ContextAction,
        proxy_string: &str,
    ) -> Result<()> {
        let cx = self.cx().session(session);
        let prx = self.context_callback_proxy(proxy_string).await?;
        let (act, text, flags) = (
            action.action.clone(),
            action.text.clone(),
            action.contexts.bits(),
        );
        call!(self, cx, |prx_srv, ctx| prx_srv.add_context_callback(
            session.get(),
            &act,
            &text,
            &prx,
            flags,
            ctx
        ))
    }

    /// Поставить аутентификатор.
    ///
    /// Murmur будет спрашивать у него про каждый вход. Пока подписка жива —
    /// аутентификатор установлен; при снятии аутентификация возвращается базе
    /// Murmur'а.
    ///
    /// ```no_run
    /// # use mumble_ice::prelude::*;
    /// # use std::sync::Arc;
    /// struct Allow;
    /// #[async_trait::async_trait]
    /// impl Authenticator for Allow {
    ///     async fn authenticate(&self, req: AuthRequest) -> AuthResult {
    ///         if req.name_ci() == "alice" && req.password == "pw" {
    ///             AuthResult::Ok(AuthOk::new(UserId(1001)))
    ///         } else {
    ///             // Незнакомое имя — НЕ Denied: пусть Murmur смотрит свою базу.
    ///             AuthResult::FallThrough
    ///         }
    ///     }
    /// }
    /// # async fn f(srv: VirtualServer) -> mumble_ice::Result<()> {
    /// let sub = srv.set_authenticator(Arc::new(Allow)).await?;
    /// # Ok(()) }
    /// ```
    pub async fn set_authenticator(
        &self,
        auth: Arc<dyn crate::auth::Authenticator>,
    ) -> Result<crate::events::Subscription> {
        crate::auth::make_authenticator_subscription(
            self.inner.client.registry(),
            self,
            auth,
        )
        .await
    }

    pub(crate) async fn set_authenticator_proxy(&self, proxy_string: &str) -> Result<()> {
        let cx = self.cx();
        let proxy = self.inner.client.make_proxy(proxy_string).await?;
        // Прокси кастуется к базовому `ServerAuthenticator` — именно его ждёт
        // `setAuthenticator`. Наш servant отвечает `ice_isA` истиной за оба
        // интерфейса, потому что цепочка type-id теперь полная.
        let prx = slice::ServerAuthenticatorPrx::unchecked_cast(proxy)
            .await
            .map_err(|e| from_wire(e, cx))?;
        call!(self, cx, |prx_srv, ctx| prx_srv.set_authenticator(&prx, ctx))
    }

    /// Отдаёт аутентификацию обратно базе Murmur'а.
    ///
    /// В Slice нет операции «снять аутентификатор» — только «поставить», поэтому
    /// ставим прокси, который ни на что не отвечает.
    pub(crate) async fn clear_authenticator(&self) -> Result<()> {
        // Идентичность, за которой у нас нет servant'а: Murmur получит
        // ObjectNotExist и вернётся к своей базе.
        self.set_authenticator_proxy("mumble-ice-no-auth:tcp -h 127.0.0.1 -p 1")
            .await
    }

    pub(crate) async fn add_meta_callback(&self, proxy_string: &str) -> Result<()> {
        let cx = self.cx();
        let proxy = self.inner.client.make_proxy(proxy_string).await?;
        let prx = slice::MetaCallbackPrx::unchecked_cast(proxy)
            .await
            .map_err(|e| from_wire(e, cx))?;
        // MetaCallback ставится на Meta, а не на Server.
        self.inner.client.add_meta_callback(&prx).await
    }

    pub(crate) async fn remove_context_callback(&self, proxy_string: &str) -> Result<()> {
        let cx = self.cx();
        let prx = self.context_callback_proxy(proxy_string).await?;
        call!(self, cx, |prx_srv, ctx| prx_srv
            .remove_context_callback(&prx, ctx))
    }

    async fn callback_proxy(&self, proxy_string: &str) -> Result<slice::ServerCallbackPrx> {
        let proxy = self.inner.client.make_proxy(proxy_string).await?;
        slice::ServerCallbackPrx::unchecked_cast(proxy)
            .await
            .map_err(|e| from_wire(e, self.cx()))
    }

    async fn context_callback_proxy(
        &self,
        proxy_string: &str,
    ) -> Result<slice::ServerContextCallbackPrx> {
        let proxy = self.inner.client.make_proxy(proxy_string).await?;
        slice::ServerContextCallbackPrx::unchecked_cast(proxy)
            .await
            .map_err(|e| from_wire(e, self.cx()))
    }

    /// Пик одновременных запросов на соединении этого сервера.
    ///
    /// Больше единицы означает, что вызовы действительно мультиплексируются, а
    /// не выстраиваются в очередь. Нужно тестам: на локальном сервере вызовы
    /// быстрее шума таймера, поэтому конкурентность проверяется фактом.
    pub async fn max_in_flight(&self) -> usize {
        match self.inner.prx.proxy.connection().await {
            Ok(c) => c.max_in_flight(),
            Err(_) => 0,
        }
    }

    /// Escape hatch: сгенерированный прокси `Server` с готовым контекстом.
    pub async fn raw(&self) -> crate::raw::RawServer {
        crate::raw::RawServer::new(self.inner.prx.clone(), self.inner.shared.clone())
    }

    /// Удобный конструктор записи ACL для группы.
    pub fn group_subject(name: impl Into<String>) -> AclSubject {
        AclSubject::Group(name.into())
    }
}

impl std::fmt::Debug for VirtualServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualServer")
            .field("id", &self.id())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::ids::*;
    use crate::model::PasswordCheck;

    /// Сентинелы `verifyPassword` не должны протекать наружу как числа.
    #[test]
    fn password_check_sentinels() {
        let map = |id: i32| match id {
            -1 => PasswordCheck::WrongPassword,
            -2 => PasswordCheck::NoSuchUser,
            id => PasswordCheck::Ok(UserId(id)),
        };
        assert_eq!(PasswordCheck::WrongPassword, map(-1));
        assert_eq!(PasswordCheck::NoSuchUser, map(-2));
        assert_eq!(PasswordCheck::Ok(UserId(7)), map(7));

        // Обе формы отказа наружу выглядят одинаково — это осознанно.
        let to_option = |c: PasswordCheck| match c {
            PasswordCheck::Ok(id) => Some(id),
            _ => None,
        };
        assert_eq!(None, to_option(map(-1)));
        assert_eq!(None, to_option(map(-2)));
        assert_eq!(Some(UserId(7)), to_option(map(7)));
    }
}
