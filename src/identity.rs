//! Item identity, independent of display and filename.
//!
//! Formats 1–3 and 5 number items; format 4 named them by UUIDv4. Format 5
//! also tags each item with a UUIDv4 `uid`, written once and never shown by
//! default: it still names the item after a merge renumbers it, and it is
//! what a format-4 reference in a note or a commit trailer resolves to.
use std::fmt;
use std::str::FromStr;

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum Id {
    Num(u32),
    Uuid(uuid::Uuid),
}

impl Default for Id {
    fn default() -> Self {
        Self::Num(0)
    }
}

impl From<u32> for Id {
    fn from(n: u32) -> Self {
        Self::Num(n)
    }
}

impl PartialEq<u32> for Id {
    fn eq(&self, n: &u32) -> bool {
        *self == Self::Num(*n)
    }
}

impl Id {
    pub fn is_uuid(self) -> bool {
        matches!(self, Self::Uuid(_))
    }
    pub fn compact(self) -> String {
        match self {
            Self::Num(n) => n.to_string(),
            Self::Uuid(id) => id.simple().to_string(),
        }
    }
    pub fn rank(self) -> u128 {
        match self {
            Self::Num(n) => u128::from(n),
            Self::Uuid(id) => id.as_u128(),
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => n.fmt(f),
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
            .map(Self::Num)
            .map_err(|_| format!("invalid item identity: {raw}"))
    }
}

/// A tag as written: full, hyphenated or compact, and version 4.
pub fn parse_uid(raw: &str) -> Result<uuid::Uuid, String> {
    match raw.trim().parse::<Id>() {
        Ok(Id::Uuid(uid)) => Ok(uid),
        _ => Err(format!("`{raw}` is not a UUIDv4 tag")),
    }
}

/// Whether text could be the start of a tag rather than a number: eight or
/// more hex digits with at least one letter. All-digit text is always a
/// number, so a tag prefix never shadows an id.
pub fn is_uid_prefix(raw: &str) -> bool {
    (8..=32).contains(&raw.len())
        && raw.bytes().all(|b| b.is_ascii_hexdigit())
        && raw.bytes().any(|b| b.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tag_prefix_needs_a_letter_so_it_never_shadows_a_number() {
        assert!(is_uid_prefix("c7ac551e"));
        assert!(!is_uid_prefix("12345678"), "all digits is a number");
        assert!(!is_uid_prefix("c7ac551"), "too short");
        assert!(!is_uid_prefix("c7ac551g"), "not hex");
    }

    #[test]
    fn a_tag_must_be_a_whole_version_4_uuid() {
        let uid = "c7ac551e-b7f3-4da0-b7a1-94fd027ea098";
        assert_eq!(parse_uid(uid).unwrap().to_string(), uid);
        assert_eq!(
            parse_uid(&uid.replace('-', "").to_uppercase())
                .unwrap()
                .to_string(),
            uid
        );
        assert!(parse_uid("c7ac551e").is_err(), "a prefix is not a tag");
        assert!(parse_uid("00000000-0000-7000-8000-000000000000").is_err());
    }
}
