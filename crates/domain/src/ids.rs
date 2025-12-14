use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{self, Debug};
use uuid::Uuid;

mod private {
    pub trait Sealed {}
}

pub trait DomainId:
    private::Sealed + fmt::Display + fmt::Debug + Clone + Send + Sync + 'static
{
    const ENTITY_NAME: &'static str;

    fn as_uuid(&self) -> &Uuid;
    fn from_uuid(uuid: Uuid) -> Self;
}

macro_rules! define_id {
    ($(#[$meta:meta])*
    $name:ident => $entity:literal) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl private::Sealed for $name {}

        impl DomainId for $name {
            const ENTITY_NAME: &'static str = $entity;

            fn as_uuid(&self) -> &Uuid { &self.0 }
            fn from_uuid(uuid: Uuid) -> Self { Self(uuid) }
        }

        impl $name {
            #[must_use]
            pub fn new() -> Self { Self(Uuid::now_v7()) }

            #[must_use]
            pub const fn from_raw(uuid: Uuid) -> Self { Self(uuid) }

            #[must_use]
            pub const fn into_inner(self) -> Uuid { self.0 }
        }

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), &self.0.to_string()[..8])
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse()?))
            }
        }

        impl TryFrom<&str> for $name {
            type Error = uuid::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                s.parse()
            }
        }
    };
}

define_id!(ObservationId => "observation");
define_id!(ChunkId => "chunk");

// Content Hash
mod hash_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        hex::decode(&s)
            .map_err(serde::de::Error::custom)?
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid hash length"))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(#[serde(with = "hash_serde")] [u8; 32]);

impl ContentHash {
    #[must_use]
    pub fn from_bytes(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        Self(result.into())
    }

    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    #[must_use]
    pub fn from_content(content: &str) -> Self {
        Self::from_bytes(content.as_bytes())
    }
}

impl Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ContentHash").field(&self.0).finish()
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_id_is_unique() {
        let id1 = ObservationId::new();
        let id2 = ObservationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn id_roundtrips_through_string() {
        let original = ObservationId::new();
        let as_string = original.to_string();
        let parsed: ObservationId = as_string.parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn content_hash_is_deterministic() {
        let content = "test content for hashing";
        let hash1 = ContentHash::from_content(content);
        let hash2 = ContentHash::from_content(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn content_hash_differs_for_different_content() {
        let hash1 = ContentHash::from_content("content a");
        let hash2 = ContentHash::from_content("content b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn content_hash_serialization_roundtrip() {
        let hash = ContentHash::from_content("test");
        let json = serde_json::to_string(&hash).unwrap();
        let restored: ContentHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, restored);
    }
}
