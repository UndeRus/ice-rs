#![doc = " Этот файл сгенерирован из .ice — правки будут потеряны."]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(clippy::all)]
use crate::gen::ice::*;
use async_trait::async_trait;
use ice_rs::encoding::*;
use ice_rs::errors::*;
use ice_rs::iceobject::*;
use ice_rs::protocol::*;
use ice_rs::proxy::Proxy;
use ice_rs::IceDerive;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;
#[derive(Debug, Copy, Clone, TryFromPrimitive, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ChannelInfo {
    ChannelDescription = 0i32,
    ChannelPosition = 1i32,
}
impl OptionalType for ChannelInfo {
    fn optional_type() -> u8 {
        4
    }
}
impl ToBytes for ChannelInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(IceSize { size: *self as i32 }.to_bytes()?);
        Ok(bytes)
    }
}
impl FromBytes for ChannelInfo {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let enum_value = IceSize::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?.size;
        *read_bytes = *read_bytes + read;
        match ChannelInfo::try_from(enum_value) {
            Ok(enum_type) => Ok(enum_type),
            _ => Err(Box::new(ProtocolError::new(&format!(
                "Cannot convert int {} to enum",
                enum_value
            )))),
        }
    }
}
#[derive(Debug, Copy, Clone, TryFromPrimitive, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum UserInfo {
    UserName = 0i32,
    UserEmail = 1i32,
    UserComment = 2i32,
    UserHash = 3i32,
    UserPassword = 4i32,
    UserLastActive = 5i32,
    UserKDFIterations = 6i32,
}
impl OptionalType for UserInfo {
    fn optional_type() -> u8 {
        4
    }
}
impl ToBytes for UserInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(IceSize { size: *self as i32 }.to_bytes()?);
        Ok(bytes)
    }
}
impl FromBytes for UserInfo {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let enum_value = IceSize::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?.size;
        *read_bytes = *read_bytes + read;
        match UserInfo::try_from(enum_value) {
            Ok(enum_type) => Ok(enum_type),
            _ => Err(Box::new(ProtocolError::new(&format!(
                "Cannot convert int {} to enum",
                enum_value
            )))),
        }
    }
}
#[derive(Debug, Copy, Clone, TryFromPrimitive, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Dbstate {
    Normal = 0i32,
    ReadOnly = 1i32,
}
impl OptionalType for Dbstate {
    fn optional_type() -> u8 {
        4
    }
}
impl ToBytes for Dbstate {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(IceSize { size: *self as i32 }.to_bytes()?);
        Ok(bytes)
    }
}
impl FromBytes for Dbstate {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let enum_value = IceSize::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?.size;
        *read_bytes = *read_bytes + read;
        match Dbstate::try_from(enum_value) {
            Ok(enum_type) => Ok(enum_type),
            _ => Err(Box::new(ProtocolError::new(&format!(
                "Cannot convert int {} to enum",
                enum_value
            )))),
        }
    }
}
pub type NetAddress = Vec<u8>;
pub type IntList = Vec<i32>;
pub type UserMap = HashMap<i32, User>;
pub type ChannelMap = HashMap<i32, Channel>;
pub type ChannelList = Vec<Channel>;
pub type UserList = Vec<User>;
pub type GroupList = Vec<Group>;
pub type Acllist = Vec<Acl>;
pub type LogList = Vec<LogEntry>;
pub type BanList = Vec<Ban>;
pub type IdList = Vec<i32>;
pub type NameList = Vec<String>;
pub type NameMap = HashMap<i32, String>;
pub type IdMap = HashMap<String, i32>;
pub type Texture = Vec<u8>;
pub type ConfigMap = HashMap<String, String>;
pub type GroupNameList = Vec<String>;
pub type CertificateDer = Vec<u8>;
pub type CertificateList = Vec<CertificateDer>;
pub type UserInfoMap = HashMap<UserInfo, String>;
pub type ServerList = Vec<ServerPrx>;
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct User {
    pub session: i32,
    pub userid: i32,
    pub mute: bool,
    pub deaf: bool,
    pub suppress: bool,
    pub priority_speaker: bool,
    pub self_mute: bool,
    pub self_deaf: bool,
    pub recording: bool,
    pub channel: i32,
    pub name: String,
    pub onlinesecs: i32,
    pub bytespersec: i32,
    pub version: i32,
    pub version_2: i64,
    pub release: String,
    pub os: String,
    pub osversion: String,
    pub identity: String,
    pub context: String,
    pub comment: String,
    pub address: NetAddress,
    pub tcponly: bool,
    pub idlesecs: i32,
    pub udp_ping: f32,
    pub tcp_ping: f32,
}
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct TextMessage {
    pub sessions: IntList,
    pub channels: IntList,
    pub trees: IntList,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub parent: i32,
    pub links: IntList,
    pub description: String,
    pub temporary: bool,
    pub position: i32,
}
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct Group {
    pub name: String,
    pub inherited: bool,
    pub inherit: bool,
    pub inheritable: bool,
    pub add: IntList,
    pub remove: IntList,
    pub members: IntList,
}
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct Acl {
    pub apply_here: bool,
    pub apply_subs: bool,
    pub inherited: bool,
    pub userid: i32,
    pub group: String,
    pub allow: i32,
    pub deny: i32,
}
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct Ban {
    pub address: NetAddress,
    pub bits: i32,
    pub name: String,
    pub hash: String,
    pub reason: String,
    pub start: i32,
    pub duration: i32,
}
#[derive(Debug, Clone, PartialEq, IceDerive)]
pub struct LogEntry {
    pub timestamp: i32,
    pub txt: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    pub c: Channel,
    pub children: TreeList,
    pub users: UserList,
}
impl ToBytes for Tree {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        let slice_flags = SliceFlags {
            type_id: SliceFlagsTypeEncoding::StringTypeId,
            optional_members: false,
            indirection_table: false,
            slice_size: false,
            last_slice: true,
        };
        bytes.extend(1u8.to_bytes()?);
        bytes.extend(slice_flags.to_bytes()?);
        bytes.extend("::MumbleServer::Tree".to_bytes()?);
        bytes.extend(self.c.to_bytes()?);
        bytes.extend(self.children.to_bytes()?);
        bytes.extend(self.users.to_bytes()?);
        bytes.extend(255u8.to_bytes()?);
        Ok(bytes)
    }
}
impl FromBytes for Tree {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let marker = u8::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        if marker != 1 && marker != 255 {
            read = 0;
        }
        let flags = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        match flags.type_id {
            SliceFlagsTypeEncoding::StringTypeId => {
                let _slice_name =
                    String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
            }
            SliceFlagsTypeEncoding::CompactTypeId => {
                let _compact_id =
                    IceSize::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
            }
            SliceFlagsTypeEncoding::IndexTypeId => {
                let _type_index =
                    IceSize::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
            }
            SliceFlagsTypeEncoding::NoTypeId => {}
        }
        let c = Channel::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let children = TreeList::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let users = UserList::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            c: c,
            children: children,
            users: users,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
pub type TreeList = Vec<Box<Tree>>;
#[doc = " Slice: `const PermissionWrite = 0x01;`"]
#[allow(dead_code)]
pub const PERMISSION_WRITE: i32 = 0x01;
#[doc = " Slice: `const PermissionTraverse = 0x02;`"]
#[allow(dead_code)]
pub const PERMISSION_TRAVERSE: i32 = 0x02;
#[doc = " Slice: `const PermissionEnter = 0x04;`"]
#[allow(dead_code)]
pub const PERMISSION_ENTER: i32 = 0x04;
#[doc = " Slice: `const PermissionSpeak = 0x08;`"]
#[allow(dead_code)]
pub const PERMISSION_SPEAK: i32 = 0x08;
#[doc = " Slice: `const PermissionWhisper = 0x100;`"]
#[allow(dead_code)]
pub const PERMISSION_WHISPER: i32 = 0x100;
#[doc = " Slice: `const PermissionMuteDeafen = 0x10;`"]
#[allow(dead_code)]
pub const PERMISSION_MUTE_DEAFEN: i32 = 0x10;
#[doc = " Slice: `const PermissionMove = 0x20;`"]
#[allow(dead_code)]
pub const PERMISSION_MOVE: i32 = 0x20;
#[doc = " Slice: `const PermissionMakeChannel = 0x40;`"]
#[allow(dead_code)]
pub const PERMISSION_MAKE_CHANNEL: i32 = 0x40;
#[doc = " Slice: `const PermissionMakeTempChannel = 0x400;`"]
#[allow(dead_code)]
pub const PERMISSION_MAKE_TEMP_CHANNEL: i32 = 0x400;
#[doc = " Slice: `const PermissionLinkChannel = 0x80;`"]
#[allow(dead_code)]
pub const PERMISSION_LINK_CHANNEL: i32 = 0x80;
#[doc = " Slice: `const PermissionTextMessage = 0x200;`"]
#[allow(dead_code)]
pub const PERMISSION_TEXT_MESSAGE: i32 = 0x200;
#[doc = " Slice: `const PermissionKick = 0x10000;`"]
#[allow(dead_code)]
pub const PERMISSION_KICK: i32 = 0x10000;
#[doc = " Slice: `const PermissionBan = 0x20000;`"]
#[allow(dead_code)]
pub const PERMISSION_BAN: i32 = 0x20000;
#[doc = " Slice: `const PermissionRegister = 0x40000;`"]
#[allow(dead_code)]
pub const PERMISSION_REGISTER: i32 = 0x40000;
#[doc = " Slice: `const PermissionRegisterSelf = 0x80000;`"]
#[allow(dead_code)]
pub const PERMISSION_REGISTER_SELF: i32 = 0x80000;
#[doc = " Slice: `const ResetUserContent = 0x100000;`"]
#[allow(dead_code)]
pub const RESET_USER_CONTENT: i32 = 0x100000;
#[doc = " Slice: `const ContextServer = 0x01;`"]
#[allow(dead_code)]
pub const CONTEXT_SERVER: i32 = 0x01;
#[doc = " Slice: `const ContextChannel = 0x02;`"]
#[allow(dead_code)]
pub const CONTEXT_CHANNEL: i32 = 0x02;
#[doc = " Slice: `const ContextUser = 0x04;`"]
#[allow(dead_code)]
pub const CONTEXT_USER: i32 = 0x04;
#[derive(Debug)]
pub struct ServerException {}
impl std::fmt::Display for ServerException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerException")
    }
}
impl std::error::Error for ServerException {}
impl ToBytes for ServerException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        Ok(bytes)
    }
}
impl FromBytes for ServerException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {};
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InternalErrorException {
    pub extends: ServerException,
}
impl std::fmt::Display for InternalErrorException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InternalErrorException")
    }
}
impl std::error::Error for InternalErrorException {}
impl ToBytes for InternalErrorException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InternalErrorException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidSessionException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidSessionException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidSessionException")
    }
}
impl std::error::Error for InvalidSessionException {}
impl ToBytes for InvalidSessionException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidSessionException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidChannelException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidChannelException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidChannelException")
    }
}
impl std::error::Error for InvalidChannelException {}
impl ToBytes for InvalidChannelException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidChannelException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidServerException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidServerException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidServerException")
    }
}
impl std::error::Error for InvalidServerException {}
impl ToBytes for InvalidServerException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidServerException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct ServerBootedException {
    pub extends: ServerException,
}
impl std::fmt::Display for ServerBootedException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerBootedException")
    }
}
impl std::error::Error for ServerBootedException {}
impl ToBytes for ServerBootedException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for ServerBootedException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct ServerFailureException {
    pub extends: ServerException,
}
impl std::fmt::Display for ServerFailureException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerFailureException")
    }
}
impl std::error::Error for ServerFailureException {}
impl ToBytes for ServerFailureException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for ServerFailureException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidUserException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidUserException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidUserException")
    }
}
impl std::error::Error for InvalidUserException {}
impl ToBytes for InvalidUserException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidUserException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidTextureException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidTextureException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidTextureException")
    }
}
impl std::error::Error for InvalidTextureException {}
impl ToBytes for InvalidTextureException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidTextureException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidCallbackException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidCallbackException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidCallbackException")
    }
}
impl std::error::Error for InvalidCallbackException {}
impl ToBytes for InvalidCallbackException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidCallbackException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidSecretException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidSecretException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidSecretException")
    }
}
impl std::error::Error for InvalidSecretException {}
impl ToBytes for InvalidSecretException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidSecretException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct NestingLimitException {
    pub extends: ServerException,
}
impl std::fmt::Display for NestingLimitException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NestingLimitException")
    }
}
impl std::error::Error for NestingLimitException {}
impl ToBytes for NestingLimitException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for NestingLimitException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct WriteOnlyException {
    pub extends: ServerException,
}
impl std::fmt::Display for WriteOnlyException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WriteOnlyException")
    }
}
impl std::error::Error for WriteOnlyException {}
impl ToBytes for WriteOnlyException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for WriteOnlyException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidInputDataException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidInputDataException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidInputDataException")
    }
}
impl std::error::Error for InvalidInputDataException {}
impl ToBytes for InvalidInputDataException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidInputDataException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct InvalidListenerException {
    pub extends: ServerException,
}
impl std::fmt::Display for InvalidListenerException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidListenerException")
    }
}
impl std::error::Error for InvalidListenerException {}
impl ToBytes for InvalidListenerException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for InvalidListenerException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[derive(Debug)]
pub struct ReadOnlyModeException {
    pub extends: ServerException,
}
impl std::fmt::Display for ReadOnlyModeException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReadOnlyModeException")
    }
}
impl std::error::Error for ReadOnlyModeException {}
impl ToBytes for ReadOnlyModeException {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        {
            let base_flags = SliceFlags {
                type_id: SliceFlagsTypeEncoding::StringTypeId,
                optional_members: false,
                indirection_table: false,
                slice_size: false,
                last_slice: true,
            };
            bytes.extend(base_flags.to_bytes()?);
            bytes.extend(String::from("::MumbleServer::ServerException").to_bytes()?);
            bytes.extend(self.extends.to_bytes()?)
        };
        Ok(bytes)
    }
}
impl FromBytes for ReadOnlyModeException {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        let mut read = 0;
        let _flag = SliceFlags::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let _slice_name = String::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?;
        let obj = Self {
            extends: ServerException::from_bytes(&bytes[read as usize..bytes.len()], &mut read)?,
        };
        *read_bytes = *read_bytes + read;
        Ok(obj)
    }
}
#[async_trait]
pub trait ServerCallback: IceObject {
    async fn user_connected(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn user_disconnected(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn user_state_changed(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn user_text_message(
        &mut self,
        state: &User,
        message: &TextMessage,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn channel_created(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn channel_removed(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn channel_state_changed(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait ServerCallbackI {
    async fn user_connected(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn user_disconnected(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn user_state_changed(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn user_text_message(
        &mut self,
        state: &User,
        message: &TextMessage,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn channel_created(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn channel_removed(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn channel_state_changed(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> ();
}
pub struct ServerCallbackServer {
    server_impl: Box<dyn ServerCallbackI + Send + Sync>,
}
impl ServerCallbackServer {
    #[allow(dead_code)]
    pub fn new(server_impl: Box<dyn ServerCallbackI + Send + Sync>) -> ServerCallbackServer {
        ServerCallbackServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::ServerCallback"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for ServerCallbackServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(
                    String::from("::MumbleServer::ServerCallback").to_bytes()?,
                ),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "userConnected" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = User::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .user_connected(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "userDisconnected" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = User::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .user_disconnected(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "userStateChanged" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = User::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .user_state_changed(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "userTextMessage" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = User::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let message = TextMessage::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .user_text_message(&state, &message, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "channelCreated" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = Channel::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .channel_created(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "channelRemoved" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = Channel::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .channel_removed(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "channelStateChanged" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = Channel::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .channel_state_changed(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct ServerCallbackPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for ServerCallbackPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(String::from("::MumbleServer::ServerCallback").to_bytes()?),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl ServerCallback for ServerCallbackPrx {
    async fn user_connected(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("userConnected"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn user_disconnected(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("userDisconnected"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn user_state_changed(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("userStateChanged"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn user_text_message(
        &mut self,
        state: &User,
        message: &TextMessage,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        bytes.extend(message.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("userTextMessage"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn channel_created(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("channelCreated"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn channel_removed(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("channelRemoved"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn channel_state_changed(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("channelStateChanged"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
}
impl ServerCallbackPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for ServerCallbackPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for ServerCallbackPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(ServerCallbackPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
#[async_trait]
pub trait ServerContextCallback: IceObject {
    async fn context_action(
        &mut self,
        action: &String,
        usr: &User,
        session: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait ServerContextCallbackI {
    async fn context_action(
        &mut self,
        action: &String,
        usr: &User,
        session: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> ();
}
pub struct ServerContextCallbackServer {
    server_impl: Box<dyn ServerContextCallbackI + Send + Sync>,
}
impl ServerContextCallbackServer {
    #[allow(dead_code)]
    pub fn new(
        server_impl: Box<dyn ServerContextCallbackI + Send + Sync>,
    ) -> ServerContextCallbackServer {
        ServerContextCallbackServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::ServerContextCallback"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for ServerContextCallbackServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(
                    String::from("::MumbleServer::ServerContextCallback").to_bytes()?,
                ),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "contextAction" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let action = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let usr = User::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .context_action(
                        &action,
                        &usr,
                        session,
                        channelid,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct ServerContextCallbackPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for ServerContextCallbackPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(
                    String::from("::MumbleServer::ServerContextCallback").to_bytes()?,
                ),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl ServerContextCallback for ServerContextCallbackPrx {
    async fn context_action(
        &mut self,
        action: &String,
        usr: &User,
        session: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(action.to_bytes()?);
        bytes.extend(usr.to_bytes()?);
        bytes.extend(session.to_bytes()?);
        bytes.extend(channelid.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("contextAction"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
}
impl ServerContextCallbackPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for ServerContextCallbackPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for ServerContextCallbackPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(ServerContextCallbackPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
#[async_trait]
pub trait ServerAuthenticator: IceObject {
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut GroupNameList,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_info(
        &mut self,
        id: i32,
        info: &mut UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn name_to_id(
        &mut self,
        name: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn id_to_name(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn id_to_texture(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Texture, Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait ServerAuthenticatorI {
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut GroupNameList,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn get_info(
        &mut self,
        id: i32,
        info: &mut UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> bool;
    async fn name_to_id(&mut self, name: &String, context: Option<HashMap<String, String>>) -> i32;
    async fn id_to_name(&mut self, id: i32, context: Option<HashMap<String, String>>) -> String;
    async fn id_to_texture(&mut self, id: i32, context: Option<HashMap<String, String>>)
        -> Texture;
}
pub struct ServerAuthenticatorServer {
    server_impl: Box<dyn ServerAuthenticatorI + Send + Sync>,
}
impl ServerAuthenticatorServer {
    #[allow(dead_code)]
    pub fn new(
        server_impl: Box<dyn ServerAuthenticatorI + Send + Sync>,
    ) -> ServerAuthenticatorServer {
        ServerAuthenticatorServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::ServerAuthenticator"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for ServerAuthenticatorServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(
                    String::from("::MumbleServer::ServerAuthenticator").to_bytes()?,
                ),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "authenticate" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let name = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let pw = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let certificates = CertificateList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let certhash = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let certstrong = bool::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let mut newname: String = Default::default();
                let mut groups: GroupNameList = Default::default();
                let result = self
                    .server_impl
                    .authenticate(
                        &name,
                        &pw,
                        &certificates,
                        &certhash,
                        certstrong,
                        &mut newname,
                        &mut groups,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(newname.to_bytes()?);
                __reply.extend(groups.to_bytes()?);
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getInfo" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let mut info: UserInfoMap = Default::default();
                let result = self
                    .server_impl
                    .get_info(id, &mut info, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(info.to_bytes()?);
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "nameToId" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let name = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .name_to_id(&name, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "idToName" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .id_to_name(id, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "idToTexture" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .id_to_texture(id, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct ServerAuthenticatorPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for ServerAuthenticatorPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(
                    String::from("::MumbleServer::ServerAuthenticator").to_bytes()?,
                ),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl ServerAuthenticator for ServerAuthenticatorPrx {
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut GroupNameList,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(name.to_bytes()?);
        bytes.extend(pw.to_bytes()?);
        bytes.extend(certificates.to_bytes()?);
        bytes.extend(certhash.to_bytes()?);
        bytes.extend(certstrong.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("authenticate"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        *newname = String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *groups = GroupNameList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_info(
        &mut self,
        id: i32,
        info: &mut UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getInfo"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        *info = UserInfoMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        bool::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn name_to_id(
        &mut self,
        name: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(name.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("nameToId"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn id_to_name(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("idToName"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn id_to_texture(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Texture, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("idToTexture"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        Texture::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
}
impl ServerAuthenticatorPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for ServerAuthenticatorPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for ServerAuthenticatorPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(ServerAuthenticatorPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
#[async_trait]
pub trait ServerUpdatingAuthenticator: IceObject {
    async fn register_user(
        &mut self,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn unregister_user(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_registered_users(
        &mut self,
        filter: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<NameMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_info(
        &mut self,
        id: i32,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_texture(
        &mut self,
        id: i32,
        tex: &Texture,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut GroupNameList,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_info(
        &mut self,
        id: i32,
        info: &mut UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn name_to_id(
        &mut self,
        name: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn id_to_name(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn id_to_texture(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Texture, Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait ServerUpdatingAuthenticatorI {
    async fn register_user(
        &mut self,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn unregister_user(&mut self, id: i32, context: Option<HashMap<String, String>>) -> i32;
    async fn get_registered_users(
        &mut self,
        filter: &String,
        context: Option<HashMap<String, String>>,
    ) -> NameMap;
    async fn set_info(
        &mut self,
        id: i32,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn set_texture(
        &mut self,
        id: i32,
        tex: &Texture,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut GroupNameList,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn get_info(
        &mut self,
        id: i32,
        info: &mut UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> bool;
    async fn name_to_id(&mut self, name: &String, context: Option<HashMap<String, String>>) -> i32;
    async fn id_to_name(&mut self, id: i32, context: Option<HashMap<String, String>>) -> String;
    async fn id_to_texture(&mut self, id: i32, context: Option<HashMap<String, String>>)
        -> Texture;
}
pub struct ServerUpdatingAuthenticatorServer {
    server_impl: Box<dyn ServerUpdatingAuthenticatorI + Send + Sync>,
}
impl ServerUpdatingAuthenticatorServer {
    #[allow(dead_code)]
    pub fn new(
        server_impl: Box<dyn ServerUpdatingAuthenticatorI + Send + Sync>,
    ) -> ServerUpdatingAuthenticatorServer {
        ServerUpdatingAuthenticatorServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::ServerUpdatingAuthenticator"),
            String::from("::MumbleServer::ServerAuthenticator"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for ServerUpdatingAuthenticatorServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(
                    String::from("::MumbleServer::ServerUpdatingAuthenticator").to_bytes()?,
                ),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "registerUser" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let info = UserInfoMap::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .register_user(&info, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "unregisterUser" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .unregister_user(id, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getRegisteredUsers" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let filter = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_registered_users(&filter, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setInfo" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let info = UserInfoMap::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_info(id, &info, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setTexture" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let tex = Texture::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_texture(id, &tex, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "authenticate" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let name = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let pw = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let certificates = CertificateList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let certhash = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let certstrong = bool::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let mut newname: String = Default::default();
                let mut groups: GroupNameList = Default::default();
                let result = self
                    .server_impl
                    .authenticate(
                        &name,
                        &pw,
                        &certificates,
                        &certhash,
                        certstrong,
                        &mut newname,
                        &mut groups,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(newname.to_bytes()?);
                __reply.extend(groups.to_bytes()?);
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getInfo" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let mut info: UserInfoMap = Default::default();
                let result = self
                    .server_impl
                    .get_info(id, &mut info, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(info.to_bytes()?);
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "nameToId" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let name = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .name_to_id(&name, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "idToName" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .id_to_name(id, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "idToTexture" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .id_to_texture(id, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct ServerUpdatingAuthenticatorPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for ServerUpdatingAuthenticatorPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(
                    String::from("::MumbleServer::ServerUpdatingAuthenticator").to_bytes()?,
                ),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl ServerUpdatingAuthenticator for ServerUpdatingAuthenticatorPrx {
    async fn register_user(
        &mut self,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(info.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("registerUser"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn unregister_user(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("unregisterUser"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_registered_users(
        &mut self,
        filter: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<NameMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(filter.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getRegisteredUsers"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        NameMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_info(
        &mut self,
        id: i32,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        bytes.extend(info.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("setInfo"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_texture(
        &mut self,
        id: i32,
        tex: &Texture,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        bytes.extend(tex.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("setTexture"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn authenticate(
        &mut self,
        name: &String,
        pw: &String,
        certificates: &CertificateList,
        certhash: &String,
        certstrong: bool,
        newname: &mut String,
        groups: &mut GroupNameList,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(name.to_bytes()?);
        bytes.extend(pw.to_bytes()?);
        bytes.extend(certificates.to_bytes()?);
        bytes.extend(certhash.to_bytes()?);
        bytes.extend(certstrong.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("authenticate"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        *newname = String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *groups = GroupNameList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_info(
        &mut self,
        id: i32,
        info: &mut UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getInfo"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        *info = UserInfoMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        bool::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn name_to_id(
        &mut self,
        name: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(name.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("nameToId"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn id_to_name(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("idToName"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn id_to_texture(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Texture, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("idToTexture"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        Texture::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
}
impl ServerUpdatingAuthenticatorPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for ServerUpdatingAuthenticatorPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for ServerUpdatingAuthenticatorPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(ServerUpdatingAuthenticatorPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
#[async_trait]
pub trait Server: IceObject {
    async fn is_running(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn start(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn stop(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn id(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn add_callback(
        &mut self,
        cb: &ServerCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn remove_callback(
        &mut self,
        cb: &ServerCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn set_authenticator(
        &mut self,
        auth: &ServerAuthenticatorPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_conf(
        &mut self,
        key: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_all_conf(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ConfigMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_conf(
        &mut self,
        key: &String,
        value: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn set_superuser_password(
        &mut self,
        pw: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_log(
        &mut self,
        first: i32,
        last: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<LogList, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_log_len(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_users(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<UserMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_channels(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ChannelMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_certificate_list(
        &mut self,
        session: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<CertificateList, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_tree(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<Tree, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_bans(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<BanList, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_bans(
        &mut self,
        bans: &BanList,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn kick_user(
        &mut self,
        session: i32,
        reason: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_state(
        &mut self,
        session: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_state(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn send_message(
        &mut self,
        session: i32,
        text: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn has_permission(
        &mut self,
        session: i32,
        channelid: i32,
        perm: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn effective_permissions(
        &mut self,
        session: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn add_context_callback(
        &mut self,
        session: i32,
        action: &String,
        text: &String,
        cb: &ServerContextCallbackPrx,
        ctx: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn remove_context_callback(
        &mut self,
        cb: &ServerContextCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_channel_state(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Channel, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_channel_state(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn remove_channel(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn add_channel(
        &mut self,
        name: &String,
        parent: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn send_message_channel(
        &mut self,
        channelid: i32,
        tree: bool,
        text: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_acl(
        &mut self,
        channelid: i32,
        acls: &mut Acllist,
        groups: &mut GroupList,
        inherit: &mut bool,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn set_acl(
        &mut self,
        channelid: i32,
        acls: &Acllist,
        groups: &GroupList,
        inherit: bool,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn add_user_to_group(
        &mut self,
        channelid: i32,
        session: i32,
        group: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn remove_user_from_group(
        &mut self,
        channelid: i32,
        session: i32,
        group: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn redirect_whisper_group(
        &mut self,
        session: i32,
        source: &String,
        target: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_user_names(
        &mut self,
        ids: &IdList,
        context: Option<HashMap<String, String>>,
    ) -> Result<NameMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_user_ids(
        &mut self,
        names: &NameList,
        context: Option<HashMap<String, String>>,
    ) -> Result<IdMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn register_user(
        &mut self,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn unregister_user(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn update_registration(
        &mut self,
        userid: i32,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_registration(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<UserInfoMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_registered_users(
        &mut self,
        filter: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<NameMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn verify_password(
        &mut self,
        name: &String,
        pw: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_texture(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Texture, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_texture(
        &mut self,
        userid: i32,
        tex: &Texture,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_uptime(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn update_certificate(
        &mut self,
        certificate: &String,
        private_key: &String,
        passphrase: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn start_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn stop_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn is_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_listening_channels(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<IntList, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_listening_users(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<IntList, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_listener_volume_adjustment(
        &mut self,
        channelid: i32,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<f32, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_listener_volume_adjustment(
        &mut self,
        channelid: i32,
        userid: i32,
        volume_adjustment: f32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn send_welcome_message(
        &mut self,
        receiver_user_i_ds: &IdList,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait ServerI {
    async fn is_running(&mut self, context: Option<HashMap<String, String>>) -> bool;
    async fn start(&mut self, context: Option<HashMap<String, String>>) -> ();
    async fn stop(&mut self, context: Option<HashMap<String, String>>) -> ();
    async fn delete(&mut self, context: Option<HashMap<String, String>>) -> ();
    async fn id(&mut self, context: Option<HashMap<String, String>>) -> i32;
    async fn add_callback(
        &mut self,
        cb: &ServerCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn remove_callback(
        &mut self,
        cb: &ServerCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn set_authenticator(
        &mut self,
        auth: &ServerAuthenticatorPrx,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_conf(&mut self, key: &String, context: Option<HashMap<String, String>>) -> String;
    async fn get_all_conf(&mut self, context: Option<HashMap<String, String>>) -> ConfigMap;
    async fn set_conf(
        &mut self,
        key: &String,
        value: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn set_superuser_password(
        &mut self,
        pw: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_log(
        &mut self,
        first: i32,
        last: i32,
        context: Option<HashMap<String, String>>,
    ) -> LogList;
    async fn get_log_len(&mut self, context: Option<HashMap<String, String>>) -> i32;
    async fn get_users(&mut self, context: Option<HashMap<String, String>>) -> UserMap;
    async fn get_channels(&mut self, context: Option<HashMap<String, String>>) -> ChannelMap;
    async fn get_certificate_list(
        &mut self,
        session: i32,
        context: Option<HashMap<String, String>>,
    ) -> CertificateList;
    async fn get_tree(&mut self, context: Option<HashMap<String, String>>) -> Tree;
    async fn get_bans(&mut self, context: Option<HashMap<String, String>>) -> BanList;
    async fn set_bans(&mut self, bans: &BanList, context: Option<HashMap<String, String>>) -> ();
    async fn kick_user(
        &mut self,
        session: i32,
        reason: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_state(&mut self, session: i32, context: Option<HashMap<String, String>>) -> User;
    async fn set_state(&mut self, state: &User, context: Option<HashMap<String, String>>) -> ();
    async fn send_message(
        &mut self,
        session: i32,
        text: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn has_permission(
        &mut self,
        session: i32,
        channelid: i32,
        perm: i32,
        context: Option<HashMap<String, String>>,
    ) -> bool;
    async fn effective_permissions(
        &mut self,
        session: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn add_context_callback(
        &mut self,
        session: i32,
        action: &String,
        text: &String,
        cb: &ServerContextCallbackPrx,
        ctx: i32,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn remove_context_callback(
        &mut self,
        cb: &ServerContextCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_channel_state(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Channel;
    async fn set_channel_state(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn remove_channel(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn add_channel(
        &mut self,
        name: &String,
        parent: i32,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn send_message_channel(
        &mut self,
        channelid: i32,
        tree: bool,
        text: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_acl(
        &mut self,
        channelid: i32,
        acls: &mut Acllist,
        groups: &mut GroupList,
        inherit: &mut bool,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn set_acl(
        &mut self,
        channelid: i32,
        acls: &Acllist,
        groups: &GroupList,
        inherit: bool,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn add_user_to_group(
        &mut self,
        channelid: i32,
        session: i32,
        group: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn remove_user_from_group(
        &mut self,
        channelid: i32,
        session: i32,
        group: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn redirect_whisper_group(
        &mut self,
        session: i32,
        source: &String,
        target: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_user_names(
        &mut self,
        ids: &IdList,
        context: Option<HashMap<String, String>>,
    ) -> NameMap;
    async fn get_user_ids(
        &mut self,
        names: &NameList,
        context: Option<HashMap<String, String>>,
    ) -> IdMap;
    async fn register_user(
        &mut self,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn unregister_user(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn update_registration(
        &mut self,
        userid: i32,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_registration(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> UserInfoMap;
    async fn get_registered_users(
        &mut self,
        filter: &String,
        context: Option<HashMap<String, String>>,
    ) -> NameMap;
    async fn verify_password(
        &mut self,
        name: &String,
        pw: &String,
        context: Option<HashMap<String, String>>,
    ) -> i32;
    async fn get_texture(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Texture;
    async fn set_texture(
        &mut self,
        userid: i32,
        tex: &Texture,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_uptime(&mut self, context: Option<HashMap<String, String>>) -> i32;
    async fn update_certificate(
        &mut self,
        certificate: &String,
        private_key: &String,
        passphrase: &String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn start_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn stop_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn is_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> bool;
    async fn get_listening_channels(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> IntList;
    async fn get_listening_users(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> IntList;
    async fn get_listener_volume_adjustment(
        &mut self,
        channelid: i32,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> f32;
    async fn set_listener_volume_adjustment(
        &mut self,
        channelid: i32,
        userid: i32,
        volume_adjustment: f32,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn send_welcome_message(
        &mut self,
        receiver_user_i_ds: &IdList,
        context: Option<HashMap<String, String>>,
    ) -> ();
}
pub struct ServerServer {
    server_impl: Box<dyn ServerI + Send + Sync>,
}
impl ServerServer {
    #[allow(dead_code)]
    pub fn new(server_impl: Box<dyn ServerI + Send + Sync>) -> ServerServer {
        ServerServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::Server"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for ServerServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(String::from("::MumbleServer::Server").to_bytes()?),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "isRunning" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .is_running(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "start" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self.server_impl.start(Some(request.context.clone())).await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "stop" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self.server_impl.stop(Some(request.context.clone())).await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "delete" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self.server_impl.delete(Some(request.context.clone())).await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "id" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self.server_impl.id(Some(request.context.clone())).await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "addCallback" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let cb = ServerCallbackPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .add_callback(&cb, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "removeCallback" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let cb = ServerCallbackPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .remove_callback(&cb, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setAuthenticator" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let auth = ServerAuthenticatorPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_authenticator(&auth, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getConf" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let key = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_conf(&key, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getAllConf" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_all_conf(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setConf" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let key = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let value = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_conf(&key, &value, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setSuperuserPassword" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let pw = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_superuser_password(&pw, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getLog" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let first = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let last = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_log(first, last, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getLogLen" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_log_len(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getUsers" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_users(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getChannels" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_channels(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getCertificateList" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_certificate_list(session, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getTree" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_tree(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getBans" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_bans(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setBans" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let bans = BanList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_bans(&bans, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "kickUser" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let reason = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .kick_user(session, &reason, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getState" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_state(session, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setState" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = User::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_state(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "sendMessage" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let text = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .send_message(session, &text, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "hasPermission" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let perm = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .has_permission(session, channelid, perm, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "effectivePermissions" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .effective_permissions(session, channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "addContextCallback" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let action = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let text = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let cb = ServerContextCallbackPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let ctx = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .add_context_callback(
                        session,
                        &action,
                        &text,
                        &cb,
                        ctx,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "removeContextCallback" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let cb = ServerContextCallbackPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .remove_context_callback(&cb, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getChannelState" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_channel_state(channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setChannelState" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = Channel::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_channel_state(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "removeChannel" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .remove_channel(channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "addChannel" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let name = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let parent = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .add_channel(&name, parent, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "sendMessageChannel" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let tree = bool::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let text = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .send_message_channel(channelid, tree, &text, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getACL" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let mut acls: Acllist = Default::default();
                let mut groups: GroupList = Default::default();
                let mut inherit: bool = Default::default();
                let result = self
                    .server_impl
                    .get_acl(
                        channelid,
                        &mut acls,
                        &mut groups,
                        &mut inherit,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(acls.to_bytes()?);
                __reply.extend(groups.to_bytes()?);
                __reply.extend(inherit.to_bytes()?);
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setACL" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let acls = Acllist::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let groups = GroupList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let inherit = bool::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_acl(
                        channelid,
                        &acls,
                        &groups,
                        inherit,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "addUserToGroup" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let group = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .add_user_to_group(channelid, session, &group, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "removeUserFromGroup" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let group = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .remove_user_from_group(
                        channelid,
                        session,
                        &group,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "redirectWhisperGroup" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let session = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let source = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let target = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .redirect_whisper_group(
                        session,
                        &source,
                        &target,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getUserNames" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let ids = IdList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_user_names(&ids, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getUserIds" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let names = NameList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_user_ids(&names, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "registerUser" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let info = UserInfoMap::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .register_user(&info, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "unregisterUser" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .unregister_user(userid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "updateRegistration" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let info = UserInfoMap::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .update_registration(userid, &info, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getRegistration" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_registration(userid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getRegisteredUsers" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let filter = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_registered_users(&filter, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "verifyPassword" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let name = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let pw = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .verify_password(&name, &pw, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getTexture" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_texture(userid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setTexture" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let tex = Texture::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_texture(userid, &tex, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getUptime" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_uptime(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "updateCertificate" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let certificate = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let private_key = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let passphrase = String::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .update_certificate(
                        &certificate,
                        &private_key,
                        &passphrase,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "startListening" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .start_listening(userid, channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "stopListening" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .stop_listening(userid, channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "isListening" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .is_listening(userid, channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getListeningChannels" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_listening_channels(userid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getListeningUsers" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_listening_users(channelid, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getListenerVolumeAdjustment" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_listener_volume_adjustment(
                        channelid,
                        userid,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setListenerVolumeAdjustment" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let channelid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let userid = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let volume_adjustment = f32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_listener_volume_adjustment(
                        channelid,
                        userid,
                        volume_adjustment,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "sendWelcomeMessage" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let receiver_user_i_ds = IdList::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .send_welcome_message(&receiver_user_i_ds, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct ServerPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for ServerPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(String::from("::MumbleServer::Server").to_bytes()?),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl Server for ServerPrx {
    async fn is_running(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("isRunning"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn start(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("start"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn stop(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("stop"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn delete(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("delete"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn id(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("id"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn add_callback(
        &mut self,
        cb: &ServerCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(cb.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("addCallback"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn remove_callback(
        &mut self,
        cb: &ServerCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(cb.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("removeCallback"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn set_authenticator(
        &mut self,
        auth: &ServerAuthenticatorPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(auth.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setAuthenticator"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_conf(
        &mut self,
        key: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(key.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getConf"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_all_conf(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ConfigMap, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getAllConf"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        ConfigMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_conf(
        &mut self,
        key: &String,
        value: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(key.to_bytes()?);
        bytes.extend(value.to_bytes()?);
        self.proxy
            .dispatch::<InvalidSecretException>(
                &String::from("setConf"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn set_superuser_password(
        &mut self,
        pw: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(pw.to_bytes()?);
        self.proxy
            .dispatch::<InvalidSecretException>(
                &String::from("setSuperuserPassword"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_log(
        &mut self,
        first: i32,
        last: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<LogList, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(first.to_bytes()?);
        bytes.extend(last.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getLog"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        LogList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_log_len(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getLogLen"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_users(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<UserMap, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getUsers"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        UserMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_channels(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ChannelMap, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getChannels"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        ChannelMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_certificate_list(
        &mut self,
        session: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<CertificateList, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getCertificateList"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        CertificateList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_tree(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<Tree, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getTree"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        Tree::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_bans(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<BanList, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getBans"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        BanList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_bans(
        &mut self,
        bans: &BanList,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(bans.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setBans"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn kick_user(
        &mut self,
        session: i32,
        reason: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        bytes.extend(reason.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("kickUser"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_state(
        &mut self,
        session: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getState"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        User::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_state(
        &mut self,
        state: &User,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setState"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn send_message(
        &mut self,
        session: i32,
        text: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        bytes.extend(text.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("sendMessage"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn has_permission(
        &mut self,
        session: i32,
        channelid: i32,
        perm: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(perm.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("hasPermission"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn effective_permissions(
        &mut self,
        session: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        bytes.extend(channelid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("effectivePermissions"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn add_context_callback(
        &mut self,
        session: i32,
        action: &String,
        text: &String,
        cb: &ServerContextCallbackPrx,
        ctx: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        bytes.extend(action.to_bytes()?);
        bytes.extend(text.to_bytes()?);
        bytes.extend(cb.to_bytes()?);
        bytes.extend(ctx.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("addContextCallback"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn remove_context_callback(
        &mut self,
        cb: &ServerContextCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(cb.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("removeContextCallback"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_channel_state(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Channel, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getChannelState"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        Channel::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_channel_state(
        &mut self,
        state: &Channel,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setChannelState"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn remove_channel(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("removeChannel"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn add_channel(
        &mut self,
        name: &String,
        parent: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(name.to_bytes()?);
        bytes.extend(parent.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("addChannel"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn send_message_channel(
        &mut self,
        channelid: i32,
        tree: bool,
        text: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(tree.to_bytes()?);
        bytes.extend(text.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("sendMessageChannel"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_acl(
        &mut self,
        channelid: i32,
        acls: &mut Acllist,
        groups: &mut GroupList,
        inherit: &mut bool,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getACL"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        *acls = Acllist::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *groups = GroupList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *inherit = bool::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        Ok(())
    }
    async fn set_acl(
        &mut self,
        channelid: i32,
        acls: &Acllist,
        groups: &GroupList,
        inherit: bool,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(acls.to_bytes()?);
        bytes.extend(groups.to_bytes()?);
        bytes.extend(inherit.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setACL"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn add_user_to_group(
        &mut self,
        channelid: i32,
        session: i32,
        group: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(session.to_bytes()?);
        bytes.extend(group.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("addUserToGroup"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn remove_user_from_group(
        &mut self,
        channelid: i32,
        session: i32,
        group: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(session.to_bytes()?);
        bytes.extend(group.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("removeUserFromGroup"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn redirect_whisper_group(
        &mut self,
        session: i32,
        source: &String,
        target: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(session.to_bytes()?);
        bytes.extend(source.to_bytes()?);
        bytes.extend(target.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("redirectWhisperGroup"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_user_names(
        &mut self,
        ids: &IdList,
        context: Option<HashMap<String, String>>,
    ) -> Result<NameMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(ids.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getUserNames"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        NameMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_user_ids(
        &mut self,
        names: &NameList,
        context: Option<HashMap<String, String>>,
    ) -> Result<IdMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(names.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getUserIds"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        IdMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn register_user(
        &mut self,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(info.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("registerUser"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn unregister_user(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("unregisterUser"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn update_registration(
        &mut self,
        userid: i32,
        info: &UserInfoMap,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        bytes.extend(info.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("updateRegistration"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_registration(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<UserInfoMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getRegistration"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        UserInfoMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_registered_users(
        &mut self,
        filter: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<NameMap, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(filter.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getRegisteredUsers"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        NameMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn verify_password(
        &mut self,
        name: &String,
        pw: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(name.to_bytes()?);
        bytes.extend(pw.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("verifyPassword"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_texture(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<Texture, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getTexture"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        Texture::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_texture(
        &mut self,
        userid: i32,
        tex: &Texture,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        bytes.extend(tex.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setTexture"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_uptime(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getUptime"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn update_certificate(
        &mut self,
        certificate: &String,
        private_key: &String,
        passphrase: &String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(certificate.to_bytes()?);
        bytes.extend(private_key.to_bytes()?);
        bytes.extend(passphrase.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("updateCertificate"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn start_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        bytes.extend(channelid.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("startListening"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn stop_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        bytes.extend(channelid.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("stopListening"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn is_listening(
        &mut self,
        userid: i32,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        bytes.extend(channelid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("isListening"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_listening_channels(
        &mut self,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<IntList, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(userid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getListeningChannels"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        IntList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_listening_users(
        &mut self,
        channelid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<IntList, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getListeningUsers"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        IntList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_listener_volume_adjustment(
        &mut self,
        channelid: i32,
        userid: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<f32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(userid.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<ServerBootedException>(
                &String::from("getListenerVolumeAdjustment"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        f32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_listener_volume_adjustment(
        &mut self,
        channelid: i32,
        userid: i32,
        volume_adjustment: f32,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(channelid.to_bytes()?);
        bytes.extend(userid.to_bytes()?);
        bytes.extend(volume_adjustment.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("setListenerVolumeAdjustment"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn send_welcome_message(
        &mut self,
        receiver_user_i_ds: &IdList,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(receiver_user_i_ds.to_bytes()?);
        self.proxy
            .dispatch::<ServerBootedException>(
                &String::from("sendWelcomeMessage"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
}
impl ServerPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for ServerPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for ServerPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(ServerPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
#[async_trait]
pub trait MetaCallback: IceObject {
    async fn started(
        &mut self,
        srv: &ServerPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn stopped(
        &mut self,
        srv: &ServerPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait MetaCallbackI {
    async fn started(&mut self, srv: &ServerPrx, context: Option<HashMap<String, String>>) -> ();
    async fn stopped(&mut self, srv: &ServerPrx, context: Option<HashMap<String, String>>) -> ();
}
pub struct MetaCallbackServer {
    server_impl: Box<dyn MetaCallbackI + Send + Sync>,
}
impl MetaCallbackServer {
    #[allow(dead_code)]
    pub fn new(server_impl: Box<dyn MetaCallbackI + Send + Sync>) -> MetaCallbackServer {
        MetaCallbackServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::MetaCallback"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for MetaCallbackServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(String::from("::MumbleServer::MetaCallback").to_bytes()?),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "started" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let srv = ServerPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .started(&srv, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "stopped" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let srv = ServerPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .stopped(&srv, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct MetaCallbackPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for MetaCallbackPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(String::from("::MumbleServer::MetaCallback").to_bytes()?),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl MetaCallback for MetaCallbackPrx {
    async fn started(
        &mut self,
        srv: &ServerPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(srv.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("started"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn stopped(
        &mut self,
        srv: &ServerPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(srv.to_bytes()?);
        self.proxy
            .dispatch::<ProtocolError>(
                &String::from("stopped"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
}
impl MetaCallbackPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for MetaCallbackPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for MetaCallbackPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(MetaCallbackPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
#[async_trait]
pub trait Meta: IceObject {
    async fn get_server(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerPrx, Box<dyn std::error::Error + Send + Sync>>;
    async fn new_server(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerPrx, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_booted_servers(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerList, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_all_servers(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerList, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_default_conf(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ConfigMap, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_version(
        &mut self,
        major: &mut i32,
        minor: &mut i32,
        patch: &mut i32,
        text: &mut String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn add_callback(
        &mut self,
        cb: &MetaCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn remove_callback(
        &mut self,
        cb: &MetaCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_uptime(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_slice(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_slice_checksums(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<super::ice::SliceChecksumDict, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_assumed_database_state(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<Dbstate, Box<dyn std::error::Error + Send + Sync>>;
    async fn set_assumed_database_state(
        &mut self,
        state: &Dbstate,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait MetaI {
    async fn get_server(&mut self, id: i32, context: Option<HashMap<String, String>>) -> ServerPrx;
    async fn new_server(&mut self, context: Option<HashMap<String, String>>) -> ServerPrx;
    async fn get_booted_servers(&mut self, context: Option<HashMap<String, String>>) -> ServerList;
    async fn get_all_servers(&mut self, context: Option<HashMap<String, String>>) -> ServerList;
    async fn get_default_conf(&mut self, context: Option<HashMap<String, String>>) -> ConfigMap;
    async fn get_version(
        &mut self,
        major: &mut i32,
        minor: &mut i32,
        patch: &mut i32,
        text: &mut String,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn add_callback(
        &mut self,
        cb: &MetaCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn remove_callback(
        &mut self,
        cb: &MetaCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> ();
    async fn get_uptime(&mut self, context: Option<HashMap<String, String>>) -> i32;
    async fn get_slice(&mut self, context: Option<HashMap<String, String>>) -> String;
    async fn get_slice_checksums(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> super::ice::SliceChecksumDict;
    async fn get_assumed_database_state(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Dbstate;
    async fn set_assumed_database_state(
        &mut self,
        state: &Dbstate,
        context: Option<HashMap<String, String>>,
    ) -> ();
}
pub struct MetaServer {
    server_impl: Box<dyn MetaI + Send + Sync>,
}
impl MetaServer {
    #[allow(dead_code)]
    pub fn new(server_impl: Box<dyn MetaI + Send + Sync>) -> MetaServer {
        MetaServer { server_impl }
    }
    #[doc = r" Отвечает по всей цепочке наследования, а не только по"]
    #[doc = r" собственному type-id: иначе `checkedCast` к базовому"]
    #[doc = r" интерфейсу со стороны пира проваливается."]
    async fn ice_is_a(&self, param: &str) -> bool {
        Self::ice_type_ids().iter().any(|t| t == param)
    }
    #[doc = r" Slice type-id'ы объекта, от самого производного к"]
    #[doc = r" `::Ice::Object`."]
    #[allow(dead_code)]
    pub fn ice_type_ids() -> Vec<String> {
        vec![
            String::from("::MumbleServer::Meta"),
            String::from("::Ice::Object"),
        ]
    }
    #[doc = r" Оборачивает в `Servant`, пригодный для регистрации в адаптере."]
    #[allow(dead_code)]
    pub fn into_servant(self) -> std::sync::Arc<dyn ice_rs::iceobject::Servant> {
        ice_rs::adapter::LegacyServant::new(Box::new(self), Self::ice_type_ids())
    }
}
#[async_trait]
impl IceObjectServer for MetaServer {
    async fn handle_request(
        &mut self,
        request: &RequestData,
    ) -> Result<ReplyData, Box<dyn std::error::Error + Sync + Send>> {
        match request.operation.as_ref() {
            "ice_ping" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::empty(),
            }),
            "ice_id" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(String::from("::MumbleServer::Meta").to_bytes()?),
            }),
            "ice_ids" => Ok(ReplyData {
                request_id: request.request_id,
                status: 0,
                body: Encapsulation::from(Self::ice_type_ids().to_bytes()?),
            }),
            "ice_isA" => {
                let buf = ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read = 0;
                let param = String::from_bytes(&buf, &mut read)?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(self.ice_is_a(&param).await.to_bytes()?),
                })
            }
            "getServer" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let id = i32::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .get_server(id, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "newServer" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .new_server(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getBootedServers" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_booted_servers(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getAllServers" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_all_servers(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getDefaultConf" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_default_conf(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getVersion" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let mut major: i32 = Default::default();
                let mut minor: i32 = Default::default();
                let mut patch: i32 = Default::default();
                let mut text: String = Default::default();
                let result = self
                    .server_impl
                    .get_version(
                        &mut major,
                        &mut minor,
                        &mut patch,
                        &mut text,
                        Some(request.context.clone()),
                    )
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(major.to_bytes()?);
                __reply.extend(minor.to_bytes()?);
                __reply.extend(patch.to_bytes()?);
                __reply.extend(text.to_bytes()?);
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "addCallback" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let cb = MetaCallbackPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .add_callback(&cb, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "removeCallback" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let cb = MetaCallbackPrx::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .remove_callback(&cb, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getUptime" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_uptime(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getSlice" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_slice(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getSliceChecksums" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_slice_checksums(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "getAssumedDatabaseState" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let _ = __req_param_buf.len();
                let result = self
                    .server_impl
                    .get_assumed_database_state(Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let result = wrapped_result.to_bytes()?;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            "setAssumedDatabaseState" => {
                let __req_param_buf =
                    ice_rs::protocol::peel_slice_param_payload(&request.params.data);
                let mut read_bytes = 0;
                let state = Dbstate::from_bytes(
                    &__req_param_buf[read_bytes as usize..__req_param_buf.len()],
                    &mut read_bytes,
                )?;
                let result = self
                    .server_impl
                    .set_assumed_database_state(&state, Some(request.context.clone()))
                    .await;
                let wrapped_result = result;
                let mut __reply: Vec<u8> = Vec::new();
                __reply.extend(wrapped_result.to_bytes()?);
                let result = __reply;
                Ok(ReplyData {
                    request_id: request.request_id,
                    status: 0,
                    body: Encapsulation::from(result),
                })
            }
            _ => Err(Box::new(ProtocolError::new("Operation not found"))),
        }
    }
}
pub struct MetaPrx {
    pub proxy: Proxy,
}
#[async_trait]
impl IceObject for MetaPrx {
    async fn ice_ping(&mut self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        self.proxy
            .dispatch::<ProtocolError>(&String::from("ice_ping"), 1, &Encapsulation::empty(), None)
            .await?;
        Ok(())
    }
    async fn ice_is_a(&mut self) -> Result<bool, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("ice_isA"),
                1,
                &Encapsulation::from(String::from("::MumbleServer::Meta").to_bytes()?),
                None,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        bool::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_id(&mut self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_id"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(&reply.body.data, &mut read_bytes)
    }
    async fn ice_ids(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(&String::from("ice_ids"), 1, &Encapsulation::empty(), None)
            .await?;
        let mut read_bytes: i32 = 0;
        Vec::from_bytes(&reply.body.data, &mut read_bytes)
    }
}
#[async_trait]
impl Meta for MetaPrx {
    async fn get_server(
        &mut self,
        id: i32,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerPrx, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_bytes()?);
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getServer"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        let proxy_data = ProxyData::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        let proxy_string = format!(
            "{}:{} -h {} -p {}",
            proxy_data.identity_string(),
            if proxy_data.secure { "ssl" } else { "tcp" },
            self.proxy.host,
            self.proxy.port
        );
        let mut comm = ice_rs::communicator::Communicator::new().await?;
        let proxy = comm.string_to_proxy(&proxy_string).await?;
        ServerPrx::unchecked_cast(proxy).await
    }
    async fn new_server(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerPrx, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("newServer"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        let proxy_data = ProxyData::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        let proxy_string = format!(
            "{}:{} -h {} -p {}",
            proxy_data.identity_string(),
            if proxy_data.secure { "ssl" } else { "tcp" },
            self.proxy.host,
            self.proxy.port
        );
        let mut comm = ice_rs::communicator::Communicator::new().await?;
        let proxy = comm.string_to_proxy(&proxy_string).await?;
        ServerPrx::unchecked_cast(proxy).await
    }
    async fn get_booted_servers(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerList, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getBootedServers"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        ServerList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_all_servers(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ServerList, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getAllServers"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        ServerList::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_default_conf(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<ConfigMap, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getDefaultConf"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        ConfigMap::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_version(
        &mut self,
        major: &mut i32,
        minor: &mut i32,
        patch: &mut i32,
        text: &mut String,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getVersion"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        *major = i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *minor = i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *patch = i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        *text = String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )?;
        Ok(())
    }
    async fn add_callback(
        &mut self,
        cb: &MetaCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(cb.to_bytes()?);
        self.proxy
            .dispatch::<InvalidCallbackException>(
                &String::from("addCallback"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn remove_callback(
        &mut self,
        cb: &MetaCallbackPrx,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(cb.to_bytes()?);
        self.proxy
            .dispatch::<InvalidCallbackException>(
                &String::from("removeCallback"),
                0u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
    async fn get_uptime(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getUptime"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        i32::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_slice(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getSlice"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        String::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_slice_checksums(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<super::ice::SliceChecksumDict, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<ProtocolError>(
                &String::from("getSliceChecksums"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        super::ice::SliceChecksumDict::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn get_assumed_database_state(
        &mut self,
        context: Option<HashMap<String, String>>,
    ) -> Result<Dbstate, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = Vec::new();
        let reply = self
            .proxy
            .dispatch::<InvalidSecretException>(
                &String::from("getAssumedDatabaseState"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        let mut read_bytes: i32 = 0;
        Dbstate::from_bytes(
            &reply.body.data[read_bytes as usize..reply.body.data.len()],
            &mut read_bytes,
        )
    }
    async fn set_assumed_database_state(
        &mut self,
        state: &Dbstate,
        context: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        bytes.extend(state.to_bytes()?);
        self.proxy
            .dispatch::<InvalidSecretException>(
                &String::from("setAssumedDatabaseState"),
                2u8,
                &Encapsulation::from(bytes),
                context,
            )
            .await?;
        Ok(())
    }
}
impl MetaPrx {
    #[allow(dead_code)]
    pub async fn unchecked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self { proxy: proxy })
    }
    #[allow(dead_code)]
    pub async fn checked_cast(
        proxy: Proxy,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut my_proxy = Self::unchecked_cast(proxy).await?;
        if !my_proxy.ice_is_a().await? {
            return Err(Box::new(ProtocolError::new("ice_is_a() failed")));
        }
        Ok(my_proxy)
    }
}
impl ice_rs::encoding::ToBytes for MetaPrx {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.proxy.to_bytes()
    }
}
impl ice_rs::encoding::FromBytes for MetaPrx {
    fn from_bytes(
        bytes: &[u8],
        read_bytes: &mut i32,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized,
    {
        Ok(MetaPrx {
            proxy: ice_rs::proxy::Proxy::from_bytes(bytes, read_bytes)?,
        })
    }
}
