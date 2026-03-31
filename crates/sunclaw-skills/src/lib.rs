use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub required_tools: Vec<String>,
    pub risk_level: RiskLevel,
}

impl SkillManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name must not be empty".to_string());
        }

        if self.version.trim().is_empty() {
            return Err("version must not be empty".to_string());
        }

        if self
            .required_tools
            .iter()
            .any(|tool| tool.trim().is_empty())
        {
            return Err("required_tools contains empty tool name".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use crate::{RiskLevel, SkillManifest};

    #[test]
    fn validates_manifest() {
        let manifest = SkillManifest {
            name: "web-research".to_string(),
            version: "0.1.0".to_string(),
            description: "Research from web sources".to_string(),
            required_tools: vec!["search".to_string()],
            risk_level: RiskLevel::Medium,
        };

        assert!(manifest.validate().is_ok());
    }
}
