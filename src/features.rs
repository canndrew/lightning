use super::*;

const NUM_KNOWN_FEATURES: usize = 4;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GlobalFeatures {
    _private: (),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocalFeatures {
    option_data_loss_protect: FeatureFlag,
    initial_routing_sync: OptionalFeatureFlag,
    option_upfront_shutdown_script: FeatureFlag,
    gossip_queries: FeatureFlag,
}

#[derive(Debug, Fail)]
pub enum FilterFeaturesError {
    #[fail(display = "unknown required feature (bit index {})", index)]
    UnknownRequiredFeature {
        index: u16,
    },
    #[fail(display = "feature must not be required (bit index {})", index)]
    FeatureMustNotBeRequired {
        index: u16,
    },
}

#[derive(Debug, Fail)]
#[fail(display = "error filtering global features: {}", _0)]
pub struct FilterGlobalFeaturesError(#[fail(cause)] pub FilterFeaturesError);

#[derive(Debug, Fail)]
#[fail(display = "error filtering local features: {}", _0)]
pub struct FilterLocalFeaturesError(#[fail(cause)] pub FilterFeaturesError);

impl From<FilterFeaturesError> for FilterGlobalFeaturesError {
    fn from(err: FilterFeaturesError) -> FilterGlobalFeaturesError {
        FilterGlobalFeaturesError(err)
    }
}

impl From<FilterFeaturesError> for FilterLocalFeaturesError {
    fn from(err: FilterFeaturesError) -> FilterLocalFeaturesError {
        FilterLocalFeaturesError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FeatureFlag {
    No,
    Optional,
    Required,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OptionalFeatureFlag {
    No,
    Optional,
}

#[derive(Debug, Fail)]
#[fail(display = "malformed feature flag")]
pub struct MalformedFeatureFlagError;

impl FeatureFlag {
    fn from_index(bits: &[u8], index: u16) -> Result<FeatureFlag, MalformedFeatureFlagError> {
        let bits_len = bits.len();
        if index >= bits_len as u16 {
            return Ok(FeatureFlag::No);
        }
        let flags = (bits[bits_len - (1 + (index as usize) / 8)] >> (index % 8)) & 0x03;
        match flags {
            0x00 => Ok(FeatureFlag::No),
            0x01 => Ok(FeatureFlag::Required),
            0x02 => Ok(FeatureFlag::Optional),
            _ => Err(MalformedFeatureFlagError),
        }
    }

    fn to_index(self, bits: &mut [u8], index: u16) {
        let bits_len = bits.len();
        let flags = match self {
            FeatureFlag::No => 0x00,
            FeatureFlag::Required => 0x01,
            FeatureFlag::Optional => 0x02,
        };
        bits[bits_len - (1 + (index as usize) / 8)] |= flags << (index % 8);
    }

    fn try_to_optional(self) -> Option<OptionalFeatureFlag> {
        match self {
            FeatureFlag::No => Some(OptionalFeatureFlag::No),
            FeatureFlag::Required => None,
            FeatureFlag::Optional => Some(OptionalFeatureFlag::Optional),
        }
    }
}

impl OptionalFeatureFlag {
    fn to_index(self, bits: &mut [u8], index: u16) {
        let bits_len = bits.len();
        let flags = match self {
            OptionalFeatureFlag::No => 0x00,
            OptionalFeatureFlag::Optional => 0x02,
        };
        bits[bits_len - (1 + (index as usize) / 8)] |= flags << (index % 8);
    }
}

impl GlobalFeatures {
    pub fn write_to_cursor(&self, cursor: &mut WriteCursor) {
        cursor.write_u16(0);
    }
}

impl LocalFeatures {
    pub fn write_to_cursor(&self, cursor: &mut WriteCursor) {
        cursor.write_u16(1);
        let mut bytes = [0];
        let bytes = &mut bytes[..];
        self.option_data_loss_protect.to_index(bytes, 0);
        self.initial_routing_sync.to_index(bytes, 2);
        self.option_upfront_shutdown_script.to_index(bytes, 4);
        self.gossip_queries.to_index(bytes, 6);
        cursor.write_slice(bytes);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnfilteredGlobalFeatures {
    features: UnfilteredFeatures,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnfilteredLocalFeatures {
    features: UnfilteredFeatures,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct UnfilteredFeatures {
    feature_flags: SmallVec<[FeatureFlag; NUM_KNOWN_FEATURES]>,
}

impl UnfilteredFeatures {
    pub fn from_feature_flags(bytes: &[u8]) -> Result<UnfilteredFeatures, MalformedFeatureFlagError> {
        let mut feature_flags = SmallVec::new();
        let mut index = 0;
        while index < bytes.len() {
            let feature_flag = FeatureFlag::from_index(bytes, index as u16)?;
            feature_flags.push(feature_flag);
            index += 2;
        }
        Ok(UnfilteredFeatures {
            feature_flags,
        })
    }

    pub fn write_to_cursor(&self, cursor: &mut WriteCursor) {
        let num_bytes = (self.feature_flags.len() + 3) / 4;
        let mut flags: SmallVec<[u8; (NUM_KNOWN_FEATURES + 3) / 4]> = smallvec![0u8; num_bytes];
        for (half_index, feature_flag) in self.feature_flags.iter().enumerate() {
            feature_flag.to_index(&mut flags[..], half_index as u16 * 2);
        }
        cursor.write_slice(&flags);
    }

    pub fn get_index(&self, index: u16) -> FeatureFlag {
        assert_eq!(index % 2, 0);
        let half_index = index / 2;
        self.feature_flags[half_index as usize]
    }
    
    pub fn get_index_optional(&self, index: u16) -> Result<OptionalFeatureFlag, FilterFeaturesError> {
        match self.get_index(index).try_to_optional() {
            Some(flag) => Ok(flag),
            None => Err(FilterFeaturesError::FeatureMustNotBeRequired { index }),
        }
    }
}

impl UnfilteredGlobalFeatures {
    pub fn from_feature_flags(bytes: &[u8]) -> Result<UnfilteredGlobalFeatures, MalformedFeatureFlagError> {
        Ok(UnfilteredGlobalFeatures {
            features: UnfilteredFeatures::from_feature_flags(bytes)?,
        })
    }

    pub fn write_to_cursor(&self, cursor: &mut WriteCursor) {
        self.features.write_to_cursor(cursor);
    }

    pub fn filter(&self)
        -> Result<GlobalFeatures, FilterGlobalFeaturesError>
    {
        for (half_index, feature_flag) in self.features.feature_flags.iter().enumerate() {
            if *feature_flag == FeatureFlag::Required {
                Err(FilterFeaturesError::UnknownRequiredFeature {
                    index: half_index as u16 * 2,
                })?;
            }
        }
        Ok(GlobalFeatures {
            _private: (),
        })
    }
}

impl UnfilteredLocalFeatures {
    pub fn from_feature_flags(bytes: &[u8]) -> Result<UnfilteredLocalFeatures, MalformedFeatureFlagError> {
        Ok(UnfilteredLocalFeatures {
            features: UnfilteredFeatures::from_feature_flags(bytes)?,
        })
    }

    pub fn write_to_cursor(&self, cursor: &mut WriteCursor) {
        self.features.write_to_cursor(cursor);
    }

    pub fn filter(&self)
        -> Result<LocalFeatures, FilterLocalFeaturesError>
    {
        let option_data_loss_protect = self.features.get_index(0);
        let initial_routing_sync = self.features.get_index_optional(2)?;
        let option_upfront_shutdown_script = self.features.get_index(4);
        let gossip_queries = self.features.get_index(6);

        for (half_index, feature_flag) in self.features.feature_flags.iter().enumerate().skip(4) {
            if *feature_flag == FeatureFlag::Required {
                Err(FilterFeaturesError::UnknownRequiredFeature {
                    index: half_index as u16 * 2,
                })?;
            }
        }
        Ok(LocalFeatures {
            option_data_loss_protect,
            initial_routing_sync,
            option_upfront_shutdown_script,
            gossip_queries,
        })
    }
}

