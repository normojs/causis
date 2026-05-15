use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id_type!(EvidenceId);
id_type!(FactId);
id_type!(ProvenanceId);
id_type!(EntityId);
id_type!(ConflictGroupId);
id_type!(TraceId);
id_type!(SourceId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_display_is_stable() {
        let id = EvidenceId::new("evidence-hr");
        assert_eq!(id.to_string(), "evidence-hr");
        assert_eq!(id.as_str(), "evidence-hr");
    }
}
