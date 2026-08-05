//! Типизированные идентификаторы.
//!
//! Не косметика. `MumbleServer.ice` носит и `session`, и `userid` как `int`, и
//! делит операции ровно по этой границе: `kickUser`/`sendMessage`/`getState`
//! берут **session**, а `setTexture`/`unregisterUser`/`startListening` —
//! **userid**. С голыми `i32` вызов `set_texture(user.session, tex)`
//! компилируется, доезжает до Murmur и либо молча правит аватар постороннего,
//! либо падает в рантайме.
//!
//! Хуже: `startListening(userid, channelid)` и
//! `getListenerVolumeAdjustment(channelid, userid)` принимают ту же пару
//! **в противоположном порядке**.

macro_rules! id_newtype {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(pub i32);

        impl $name {
            #[inline]
            pub const fn new(v: i32) -> Self {
                Self(v)
            }
            /// Сырое значение для протокола.
            #[inline]
            pub const fn get(self) -> i32 {
                self.0
            }
        }

        impl From<i32> for $name {
            fn from(v: i32) -> Self {
                Self(v)
            }
        }

        impl From<$name> for i32 {
            fn from(v: $name) -> i32 {
                v.0
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_newtype!(
    ServerId,
    "Идентификатор виртуального сервера. `Meta::getServer(1)` — это `ServerId(1)`."
);
id_newtype!(
    SessionId,
    "Идентификатор **соединения**. Становится недействительным, как только пользователь переподключился."
);
id_newtype!(
    UserId,
    "Идентификатор **регистрации** в базе Murmur. Стабилен между подключениями."
);
id_newtype!(ChannelId, "Идентификатор канала.");

impl ChannelId {
    /// Корневой канал.
    pub const ROOT: ChannelId = ChannelId(0);
    /// «Канала нет» — так Murmur обозначает отсутствие цели.
    pub const NONE: ChannelId = ChannelId(-1);
}

impl UserId {
    pub const SUPERUSER: UserId = UserId(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Смысл ньютайпов: перепутать session и userid не должно компилироваться.
    /// Проверяем хотя бы, что типы различимы и не приводятся друг к другу молча.
    #[test]
    fn ids_are_distinct_types() {
        let s = SessionId(7);
        let u = UserId(7);
        assert_eq!(s.get(), u.get());
        // Одинаковое сырое значение, но разные типы: `assert_eq!(s, u)` не
        // скомпилировалось бы.
        assert_eq!("SessionId(7)", format!("{:?}", s));
        assert_eq!("UserId(7)", format!("{:?}", u));
        assert_eq!("7", format!("{}", s));
    }

    #[test]
    fn well_known_constants() {
        assert_eq!(0, ChannelId::ROOT.get());
        assert_eq!(-1, ChannelId::NONE.get());
        assert_eq!(0, UserId::SUPERUSER.get());
    }
}
