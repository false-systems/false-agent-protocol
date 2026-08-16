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

/// The fields a gate receipt's digest covers.
///
/// An allowlist, deliberately — not "the receipt minus `id` and
/// `receipt_digest`". A denylist silently folds in any field a producer later
/// adds, and that is not hypothetical: when Toimija attached a sauma envelope
/// to receipts, every consumer recomputing by exclusion started disagreeing
/// with the producer and receipt import broke across the stack. Both suites
/// stayed green, because each side was correct about its own definition; the
/// defect lived in the agreement between them.
///
/// Adding a field here changes every future receipt's digest and id, so it is
/// a breaking change. Adding a field to a receipt *outside* this list is not.
pub const GATE_RECEIPT_CONTENT_FIELDS: &[&str] = &[
    "gate",
    "gate_spec_digest",
    "subject",
    "started_at",
    "finished_at",
    "duration_ms",
    "exit_code",
    "stdout",
    "stderr",
    "environment",
    "evidence_grade",
];

/// Compute a gate receipt's digest from its JSON form.
///
/// Projects the receipt onto [`GATE_RECEIPT_CONTENT_FIELDS`] and digests that,
/// so a producer and a consumer cannot drift apart over which fields count.
/// Refuses a receipt missing any of them rather than digesting a partial
/// projection, which would silently produce a plausible wrong answer.
pub fn gate_receipt_digest(receipt: &Value) -> Result<String, ProtocolError> {
    let object = receipt.as_object().ok_or_else(|| {
        ProtocolError::new(
            ErrorCode::InvalidRequest,
            "gate receipt must be a JSON object",
            false,
            "false-agent-protocol",
        )
    })?;
    let mut content = Map::new();
    for field in GATE_RECEIPT_CONTENT_FIELDS {
        let value = object.get(*field).ok_or_else(|| {
            ProtocolError::new(
                ErrorCode::InvalidRequest,
                format!("gate receipt is missing the `{field}` field its digest covers"),
                false,
                "false-agent-protocol",
            )
        })?;
        content.insert((*field).to_string(), value.clone());
    }
    sha256(&Value::Object(content))
}

/// The receipt id derived from a receipt digest.
pub fn gate_receipt_id(digest: &str) -> String {
    format!(
        "gatercpt_{}",
        digest.strip_prefix("sha256:").unwrap_or(digest)
    )
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

#[cfg(test)]
mod gate_receipt_tests {
    use super::*;
    use serde_json::json;

    fn receipt() -> Value {
        json!({
            "id": "gatercpt_ignored",
            "receipt_digest": "sha256:ignored",
            "gate": "rust-format",
            "gate_spec_digest": "sha256:spec",
            "subject": {"tree_oid": "abc"},
            "started_at": "2026-08-16T00:00:00Z",
            "finished_at": "2026-08-16T00:00:01Z",
            "duration_ms": 1000,
            "exit_code": 0,
            "stdout": {"digest": "sha256:out"},
            "stderr": {"digest": "sha256:err"},
            "environment": {"operating_system": "macos"},
            "evidence_grade": "isolated_snapshot"
        })
    }

    #[test]
    fn a_field_outside_the_content_list_does_not_change_the_digest() {
        // The incident this exists to prevent: Toimija attached a sauma
        // envelope, and every consumer recomputing by exclusion disagreed.
        let bare = gate_receipt_digest(&receipt()).unwrap();
        let mut sealed = receipt();
        sealed["sauma"] = json!({"envelope": "sauma-envelope.v1", "digest": "sha256:x"});
        sealed["some_future_field"] = json!(42);
        assert_eq!(gate_receipt_digest(&sealed).unwrap(), bare);
    }

    #[test]
    fn id_and_receipt_digest_are_never_part_of_it() {
        let bare = gate_receipt_digest(&receipt()).unwrap();
        let mut relabelled = receipt();
        relabelled["id"] = json!("gatercpt_something_else");
        relabelled["receipt_digest"] = json!("sha256:something_else");
        assert_eq!(gate_receipt_digest(&relabelled).unwrap(), bare);
    }

    #[test]
    fn changing_a_covered_field_changes_the_digest() {
        let bare = gate_receipt_digest(&receipt()).unwrap();
        let mut altered = receipt();
        altered["exit_code"] = json!(1);
        assert_ne!(gate_receipt_digest(&altered).unwrap(), bare);
    }

    #[test]
    fn a_missing_covered_field_is_refused_not_digested() {
        // Digesting a partial projection would produce a plausible wrong
        // answer, which is worse than refusing.
        let mut partial = receipt();
        partial.as_object_mut().unwrap().remove("environment");
        let error = gate_receipt_digest(&partial).unwrap_err();
        assert!(format!("{error}").contains("environment"), "{error}");
    }

    #[test]
    fn key_order_in_the_input_does_not_matter() {
        let forward = gate_receipt_digest(&receipt()).unwrap();
        let reordered: Value = serde_json::from_str(&serde_json::to_string(&receipt()).unwrap())
            .map(|value: Value| {
                let object = value.as_object().unwrap().clone();
                let mut reversed = Map::new();
                for (key, value) in object.into_iter().rev() {
                    reversed.insert(key, value);
                }
                Value::Object(reversed)
            })
            .unwrap();
        assert_eq!(gate_receipt_digest(&reordered).unwrap(), forward);
    }

    #[test]
    fn the_id_is_derived_from_the_digest() {
        let digest = gate_receipt_digest(&receipt()).unwrap();
        let id = gate_receipt_id(&digest);
        assert_eq!(id, format!("gatercpt_{}", &digest[7..]));
        assert!(id.starts_with("gatercpt_"));
    }

    #[test]
    fn a_non_object_receipt_is_refused() {
        assert!(gate_receipt_digest(&json!([1, 2, 3])).is_err());
    }
}
