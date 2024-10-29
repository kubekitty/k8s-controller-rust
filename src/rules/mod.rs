// src/rules/mod.rs
use crate::types::{Finding, Rule};
use std::path::Path;
use serde::Serialize;

pub struct RulesEngine {
    rules: Vec<Rule>,
}

impl RulesEngine {
    pub fn new(rules_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rules = Self::load_rules(rules_dir)?;
        Ok(Self { rules })
    }

    fn load_rules(rules_dir: &str) -> Result<Vec<Rule>, Box<dyn std::error::Error>> {
        let mut rules = Vec::new();
        let path = Path::new(rules_dir);
        
        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                    let content = std::fs::read_to_string(entry.path())?;
                    let rule: Rule = serde_yaml::from_str(&content)?;
                    rules.push(rule);
                }
            }
        }
        
        Ok(rules)
    }

    pub fn evaluate_rules<T>(&self, _resource_type: &str, _resource: &T) -> Vec<Finding>
    where
        T: std::fmt::Debug + Serialize,
    {
        let findings = Vec::new();
        // TODO: Implement rule evaluation logic here
        findings
    }
}