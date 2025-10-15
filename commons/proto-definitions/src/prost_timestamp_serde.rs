use prost_types::Timestamp;
use serde::{self, Serializer};

pub fn serialize<S>(timestamp: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match timestamp {
        Some(ts) => serializer.serialize_str(&ts.to_string()),
        None => serializer.serialize_none(),
    }
}
