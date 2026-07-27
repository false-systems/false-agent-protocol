use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::SemanticError;

macro_rules! reference {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn parse(value: impl Into<String>) -> Result<Self, SemanticError> {
                let value = value.into();
                validate_id(&value, Self::PREFIX)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = SemanticError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

reference!(SpecRef, "spec_");
reference!(WorkRef, "work_");
reference!(ObligationRef, "obl_");
reference!(GateRef, "gate_");
reference!(GateReceiptRef, "gatercpt_");
reference!(RepositoryRef, "repo_");
reference!(RepositoryPointRef, "rpoint_");
reference!(WorkerRunRef, "run_");
reference!(ActionRef, "act_");
reference!(ToolCallRef, "tool_");
reference!(EvidenceRef, "ev_");
reference!(CompletionWitnessRef, "witness_");

fn validate_id(value: &str, prefix: &'static str) -> Result<(), SemanticError> {
    let Some(body) = value.strip_prefix(prefix) else {
        return Err(SemanticError::InvalidId {
            expected_prefix: prefix,
            observed: value.to_string(),
        });
    };
    if body.is_empty()
        || body.len() > 128
        || !body
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(SemanticError::InvalidId {
            expected_prefix: prefix,
            observed: value.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reference_enforces_its_prefix() {
        macro_rules! check {
            ($type:ty, $valid:literal) => {{
                let parsed: $type = $valid.parse().unwrap();
                assert_eq!(parsed.to_string(), $valid);
                assert!("wrong_01".parse::<$type>().is_err());
            }};
        }
        check!(SpecRef, "spec_01");
        check!(WorkRef, "work_01");
        check!(ObligationRef, "obl_01");
        check!(GateRef, "gate_01");
        check!(GateReceiptRef, "gatercpt_01");
        check!(RepositoryRef, "repo_01");
        check!(RepositoryPointRef, "rpoint_01");
        check!(WorkerRunRef, "run_01");
        check!(ActionRef, "act_01");
        check!(ToolCallRef, "tool_01");
        check!(EvidenceRef, "ev_01");
        check!(CompletionWitnessRef, "witness_01");
    }

    #[test]
    fn serde_revalidates_wire_ids() {
        let work: WorkRef = serde_json::from_str(r#""work_existing-id""#).unwrap();
        assert_eq!(
            serde_json::to_string(&work).unwrap(),
            r#""work_existing-id""#
        );
        assert!(serde_json::from_str::<WorkRef>(r#""spec_01""#).is_err());
        assert!(serde_json::from_str::<WorkRef>(r#""work_../x""#).is_err());
    }
}
