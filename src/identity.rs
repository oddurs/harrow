//! Stable item identity, independent of display and filename.
use std::fmt;
use std::str::FromStr;

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum Id {
    Legacy(u32),
    Uuid(uuid::Uuid),
}

impl Default for Id {
    fn default() -> Self {
        Self::Legacy(0)
    }
}

impl From<u32> for Id {
    fn from(n: u32) -> Self {
        Self::Legacy(n)
    }
}

impl PartialEq<u32> for Id {
    fn eq(&self, n: &u32) -> bool {
        *self == Self::Legacy(*n)
    }
}

impl Id {
    pub fn is_uuid(self) -> bool {
        matches!(self, Self::Uuid(_))
    }
    pub fn compact(self) -> String {
        match self {
            Self::Legacy(n) => n.to_string(),
            Self::Uuid(id) => id.simple().to_string(),
        }
    }
    pub fn rank(self) -> u128 {
        match self {
            Self::Legacy(n) => u128::from(n),
            Self::Uuid(id) => id.as_u128(),
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Legacy(n) => n.fmt(f),
            Self::Uuid(id) => id.hyphenated().fmt(f),
        }
    }
}

impl FromStr for Id {
    type Err = String;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let raw = raw.trim();
        if matches!(raw.len(), 32 | 36) {
            let id = uuid::Uuid::parse_str(raw).map_err(|_| format!("invalid UUID: {raw}"))?;
            if id.get_version_num() == 4 && id.get_variant() == uuid::Variant::RFC4122 {
                return Ok(Self::Uuid(id));
            }
            return Err(format!("identity must be a UUIDv4: {raw}"));
        }
        raw.strip_prefix('#')
            .unwrap_or(raw)
            .parse::<u32>()
            .map(Self::Legacy)
            .map_err(|_| format!("invalid item identity: {raw}"))
    }
}
