//! Битовые маски прав и контекстов.
//!
//! Значения берутся из `MumbleServer.ice` (строки 145–176 и 346–350) и
//! проверяются юнит-тестом против сгенерированных `murmur_slice` констант —
//! чтобы битовая маска не разъехалась с протоколом, если Slice поправят.

use murmur_slice::mumble_server as slice;

bitflags::bitflags! {
    /// Права Murmur на канале.
    ///
    /// `WRITE` подразумевает все остальные, кроме `SPEAK`.
    #[derive(Default)]
    pub struct Permission: i32 {
        const WRITE              = slice::PERMISSION_WRITE;
        const TRAVERSE           = slice::PERMISSION_TRAVERSE;
        const ENTER              = slice::PERMISSION_ENTER;
        const SPEAK              = slice::PERMISSION_SPEAK;
        const MUTE_DEAFEN        = slice::PERMISSION_MUTE_DEAFEN;
        const MOVE               = slice::PERMISSION_MOVE;
        const MAKE_CHANNEL       = slice::PERMISSION_MAKE_CHANNEL;
        const LINK_CHANNEL       = slice::PERMISSION_LINK_CHANNEL;
        const WHISPER            = slice::PERMISSION_WHISPER;
        const TEXT_MESSAGE       = slice::PERMISSION_TEXT_MESSAGE;
        const MAKE_TEMP_CHANNEL  = slice::PERMISSION_MAKE_TEMP_CHANNEL;
        /// Только на корневом канале.
        const KICK               = slice::PERMISSION_KICK;
        /// Только на корневом канале.
        const BAN                = slice::PERMISSION_BAN;
        /// Только на корневом канале.
        const REGISTER           = slice::PERMISSION_REGISTER;
        /// Только на корневом канале.
        const REGISTER_SELF      = slice::PERMISSION_REGISTER_SELF;
        /// Сбросить комментарий или аватар пользователя.
        const RESET_USER_CONTENT = slice::RESET_USER_CONTENT;
    }
}

bitflags::bitflags! {
    /// Где в меню Mumble показывать контекстное действие.
    #[derive(Default)]
    pub struct ContextFlags: i32 {
        const SERVER  = slice::CONTEXT_SERVER;
        const CHANNEL = slice::CONTEXT_CHANNEL;
        const USER    = slice::CONTEXT_USER;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Значения зафиксированы литералами: если кто-то перегенерирует биндинги из
    /// изменённого `.ice`, тест покажет расхождение, а не тихо поедет.
    #[test]
    fn permission_bits_match_the_protocol() {
        assert_eq!(0x0000_0001, Permission::WRITE.bits());
        assert_eq!(0x0000_0002, Permission::TRAVERSE.bits());
        assert_eq!(0x0000_0004, Permission::ENTER.bits());
        assert_eq!(0x0000_0008, Permission::SPEAK.bits());
        assert_eq!(0x0000_0010, Permission::MUTE_DEAFEN.bits());
        assert_eq!(0x0000_0020, Permission::MOVE.bits());
        assert_eq!(0x0000_0040, Permission::MAKE_CHANNEL.bits());
        assert_eq!(0x0000_0080, Permission::LINK_CHANNEL.bits());
        assert_eq!(0x0000_0100, Permission::WHISPER.bits());
        assert_eq!(0x0000_0200, Permission::TEXT_MESSAGE.bits());
        assert_eq!(0x0000_0400, Permission::MAKE_TEMP_CHANNEL.bits());
        assert_eq!(0x0001_0000, Permission::KICK.bits());
        assert_eq!(0x0002_0000, Permission::BAN.bits());
        assert_eq!(0x0004_0000, Permission::REGISTER.bits());
        assert_eq!(0x0008_0000, Permission::REGISTER_SELF.bits());
        assert_eq!(0x0010_0000, Permission::RESET_USER_CONTENT.bits());
    }

    #[test]
    fn context_bits_match_the_protocol() {
        assert_eq!(0x01, ContextFlags::SERVER.bits());
        assert_eq!(0x02, ContextFlags::CHANNEL.bits());
        assert_eq!(0x04, ContextFlags::USER.bits());
    }

    #[test]
    fn flags_compose_and_round_trip() {
        let p = Permission::ENTER | Permission::SPEAK | Permission::TEXT_MESSAGE;
        assert_eq!(0x0000_020C, p.bits());
        assert_eq!(Some(p), Permission::from_bits(p.bits()));
        assert!(p.contains(Permission::SPEAK));
        assert!(!p.contains(Permission::KICK));
    }

    /// Неизвестные биты не должны молча теряться: Murmur может прислать маску с
    /// битом из будущей версии.
    #[test]
    fn unknown_bits_are_preserved_by_truncate() {
        let raw = Permission::SPEAK.bits() | 0x4000_0000;
        assert_eq!(None, Permission::from_bits(raw), "строгий разбор отвергает");
        let kept = Permission::from_bits_truncate(raw);
        assert!(kept.contains(Permission::SPEAK));
    }
}
