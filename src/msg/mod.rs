pub use super::*;

mod init;

use init::*;

pub enum MsgType {
    Init = 16,
}

impl MsgType {
    pub fn from_type_tag(type_tag: u16) -> Result<MsgType, UnknownMsgType> {
        match type_tag {
            16 => Ok(MsgType::Init),
            _ => Err(UnknownMsgType { type_tag }),
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "unknown message type ({})", type_tag)]
pub struct UnknownMsgType {
    type_tag: u16,
}

impl UnknownMsgType {
    fn can_ignore(&self) -> bool {
        self.type_tag % 2 == 1
    }
}

pub enum Msg {
    Init(InitMsg),
}

#[derive(Debug, Fail)]
pub enum MsgFromBytesError {
    #[fail(display = "{}", _0)]
    UnknownMsgType(#[fail(cause)] UnknownMsgType),
    #[fail(display = "{}", _0)]
    MsgTooShort(#[fail(cause)] MsgTooShortError),
    #[fail(display = "failed to parse init msg: {}", _0)]
    Init(#[fail(cause)] InitMsgFromPayloadError),
}

impl Msg {
    pub fn msg_type(&self) -> MsgType {
        match self {
            Msg::Init { .. } => MsgType::Init,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut cursor = WriteCursor::new();
        match self {
            Msg::Init(init_msg) => {
                cursor.write_u16(MsgType::Init as u16);
                init_msg.write_to_cursor(&mut cursor);
            },
        }
        cursor.into_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Msg, MsgFromBytesError> {
        let mut cursor = ReadCursor::new(bytes);
        let type_tag = cursor.read_u16().map_err(MsgFromBytesError::MsgTooShort)?;
        let msg_type = MsgType::from_type_tag(type_tag).map_err(MsgFromBytesError::UnknownMsgType)?;
        let payload = cursor.read_to_end();
        let msg = match msg_type {
            MsgType::Init => {
                let init_msg = InitMsg::from_payload(payload).map_err(MsgFromBytesError::Init)?;
                Msg::Init(init_msg)
            },
        };
        Ok(msg)
    }
}

#[derive(Debug, Fail)]
#[fail(display = "message too short")]
pub struct MsgTooShortError;

