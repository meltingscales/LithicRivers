//! `#[serde(with = "crate::serde_pairs")]` for maps whose keys JSON can't use as object keys
//! (tuples, entities): written as a list of `[key, value]` pairs, sorted by key so the same
//! state always gives the same file.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::hash::Hash;

pub fn serialize<K, V, S>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize + Ord,
    V: Serialize,
    S: Serializer,
{
    let mut pairs: Vec<(&K, &V)> = map.iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    pairs.serialize(serializer)
}

pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Vec::<(K, V)>::deserialize(deserializer)?
        .into_iter()
        .collect())
}
