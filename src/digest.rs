use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};

use crate::error::{ErrorCode, ProtocolError};

pub fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>, ProtocolError> {
    let mut value =
        serde_json::to_value(value).map_err(|error| ProtocolError::internal(error.to_string()))?;
    canonicalize(&mut value);
    serde_json::to_vec(&value).map_err(|error| ProtocolError::internal(error.to_string()))
}

pub fn sha256(value: &impl Serialize) -> Result<String, ProtocolError> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(value)?)
    ))
}

pub fn sha256_without(value: &impl Serialize, field: &str) -> Result<String, ProtocolError> {
    let mut value =
        serde_json::to_value(value).map_err(|error| ProtocolError::internal(error.to_string()))?;
    let object = value.as_object_mut().ok_or_else(|| {
        ProtocolError::new(
            ErrorCode::InvalidRequest,
            "digest target must be a JSON object",
            false,
            "false-agent-protocol",
        )
    })?;
    object.remove(field);
    canonicalize(&mut value);
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(
            serde_json::to_vec(&value)
                .map_err(|error| ProtocolError::internal(error.to_string()))?
        )
    ))
}

fn canonicalize(value: &mut Value) {
    match value {
        Value::Array(items) => items.iter_mut().for_each(canonicalize),
        Value::Object(object) => {
            let mut sorted = Map::new();
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (key, mut value) in entries {
                canonicalize(&mut value);
                sorted.insert(key, value);
            }
            *object = sorted;
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn object_order_does_not_change_digest() {
        assert_eq!(
            sha256(&json!({"a": 1, "b": {"c": 2}})).unwrap(),
            sha256(&json!({"b": {"c": 2}, "a": 1})).unwrap()
        );
    }
}
