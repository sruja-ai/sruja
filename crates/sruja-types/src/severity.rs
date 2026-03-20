//! Severity levels for diagnostics and violations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Default)]
pub enum Severity {
    Error,
    #[default]
    Warning,
    Info,
    Hint,
}

impl Severity {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            "hint" => Ok(Severity::Hint),
            _ => Err(format!("Unknown Severity: {s}")),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Hint.to_string(), "hint");
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Info.as_str(), "info");
        assert_eq!(Severity::Hint.as_str(), "hint");
    }

    #[test]
    fn test_severity_default() {
        // The default value should be Warning due to #[default] attribute
        assert_eq!(Severity::default(), Severity::Warning);
    }

    #[test]
    fn test_severity_serde() {
        let sev = Severity::Error;
        let json = serde_json::to_string(&sev).unwrap();
        assert_eq!(json, "\"Error\"");

        let parsed: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Severity::Error);
    }

    #[test]
    fn test_severity_serde_all_variants() {
        let variants = [
            (Severity::Error, "\"Error\""),
            (Severity::Warning, "\"Warning\""),
            (Severity::Info, "\"Info\""),
            (Severity::Hint, "\"Hint\""),
        ];

        for (sev, expected_json) in variants {
            let json = serde_json::to_string(&sev).unwrap();
            assert_eq!(json, expected_json);

            let parsed: Severity = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, sev);
        }
    }
}
