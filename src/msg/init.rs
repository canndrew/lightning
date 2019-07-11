use super::*;

pub struct InitMsg {
    global_features: UnfilteredGlobalFeatures,
    local_features: UnfilteredLocalFeatures,
}

#[derive(Debug, Fail)]
pub enum InitMsgFromPayloadError {
    #[fail(display = "{}", _0)]
    PayloadTooShort(MsgTooShortError),
    #[fail(display = "failed to parse global features: {}", _0)]
    ParseGlobalFeatures(MalformedFeatureFlagError),
    #[fail(display = "failed to parse local features: {}", _0)]
    ParseLocalFeatures(MalformedFeatureFlagError),
}

impl From<MsgTooShortError> for InitMsgFromPayloadError {
    fn from(err: MsgTooShortError) -> InitMsgFromPayloadError {
        InitMsgFromPayloadError::PayloadTooShort(err)
    }
}

impl InitMsg {
    pub fn from_payload(payload: &[u8]) -> Result<InitMsg, InitMsgFromPayloadError> {
        let mut cursor = ReadCursor::new(payload);
        let global_features = {
            let gflen = cursor.read_u16()?;
            let slice = cursor.read_slice(gflen as usize)?;
            UnfilteredGlobalFeatures::from_feature_flags(slice)
                .map_err(InitMsgFromPayloadError::ParseGlobalFeatures)?
        };
        let local_features = {
            let lflen = cursor.read_u16()?;
            let slice = cursor.read_slice(lflen as usize)?;
            UnfilteredLocalFeatures::from_feature_flags(slice)
                .map_err(InitMsgFromPayloadError::ParseLocalFeatures)?
        };
        Ok(InitMsg { global_features, local_features })
    }

    pub fn write_to_cursor(&self, cursor: &mut WriteCursor) {
        self.global_features.write_to_cursor(cursor);
        self.local_features.write_to_cursor(cursor);
    }
}

