use super::*;

pub struct ReadCursor<'a> {
    pos: usize,
    bytes: &'a [u8],
}

impl<'a> ReadCursor<'a> {
    pub fn new(bytes: &'a [u8]) -> ReadCursor<'a> {
        ReadCursor {
            pos: 0,
            bytes,
        }
    }

    pub fn read_to_end(&mut self) -> &'a [u8] {
        let ret = &self.bytes[self.pos..];
        self.pos = self.bytes.len();
        ret
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], MsgTooShortError> {
        if self.pos + len > self.bytes.len() {
            return Err(MsgTooShortError);
        }
        let ret = &self.bytes[self.pos..][..len];
        self.pos += len;
        Ok(ret)
    }

    pub fn read_u16(&mut self) -> Result<u16, MsgTooShortError> {
        let ret = slice_to_array!(self.read_slice(2)?, 2);
        Ok(u16::from_be_bytes(ret))
    }
}

pub struct WriteCursor {
    bytes: BytesMut,
}

impl WriteCursor {
    pub fn new() -> WriteCursor {
        WriteCursor {
            bytes: BytesMut::new(),
        }
    }

    pub fn into_bytes(self) -> Bytes {
        self.bytes.freeze()
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        let _ = self.bytes.extend_from_slice(slice);
    }

    pub fn write_u16(&mut self, val: u16) {
        let val = u16::to_be_bytes(val);
        self.write_slice(&val[..]);
    }
}

