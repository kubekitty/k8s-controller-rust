use crate::types::{Finding, Rule};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RuleFile {
    rules: Vec<Rule>,
}

pub struct RulesEngine {
    rules: Vec<Rule>,
}

impl RulesEngine {
    pub fn new(rules_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rules = Self::load_rules(rules_dir)?;
        Ok(Self { rules })
    }

    fn load_rules(rules_dir: &str) -> Result<Vec<Rule>, Box<dyn std::error::Error>> {
        let mut all_rules = Vec::new();
        let path = Path::new(rules_dir);

        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                    let content = std::fs::read_to_string(entry.path())?;
                    let rule_file: RuleFile = serde_yaml::from_str(&content)?;
                    all_rules.extend(rule_file.rules);
                }
            }
        }

        Ok(all_rules)
    }

    pub fn evaluate_rules<T>(&self, resource_type: &str, resource: &T) -> Vec<Finding>
    where
        T: std::fmt::Debug + Serialize,
    {
        let mut findings = Vec::new();
        let resource_json = serde_json::to_value(resource).unwrap_or_default();

        for rule in &self.rules {
            if let Ok(matches) = self.evaluate_condition(&rule.condition, &resource_json) {
                if matches {
                    findings.push(Finding {
                        severity: rule.severity.clone(),
                        category: rule.category.clone(),
                        description: rule.description.clone(),
                        resource: Some(format!("{:?}", resource)),
                        namespace: self.extract_namespace(&resource_json),
                        suggestion: Some(rule.suggestion.clone()),
                    });
                }
            }
        }

        findings
    }

    fn evaluate_condition(
        &self,
        condition: &str,
        resource: &serde_json::Value,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Basic condition evaluation logic - this can be expanded
        match condition {
            "Spec.Containers[*].SecurityContext.Privileged == true" => {
                if let Some(spec) = resource.get("spec") {
                    if let Some(containers) = spec.get("containers") {
                        if let Some(containers) = containers.as_array() {
                            for container in containers {
                                if let Some(security_context) = container.get("securityContext") {
                                    if let Some(privileged) = security_context.get("privileged") {
                                        if privileged.as_bool().unwrap_or(false) {
                                            return Ok(true);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(false)
            }
            "NetworkPolicies.Count == 0" => {
                // This would need context from the cluster state
                Ok(false)
            }
            "Spec.TLS == null" => {
                if let Some(spec) = resource.get("spec") {
                    Ok(!spec.get("tls").is_some())
                } else {
                    Ok(false)
                }
            }
            // Add more condition evaluations as needed
            _ => Ok(false),
        }
    }

    fn extract_namespace(&self, resource: &serde_json::Value) -> Option<String> {
        resource
            .get("metadata")
            .and_then(|metadata| metadata.get("namespace"))
            .and_then(|ns| ns.as_str())
            .map(String::from)
    }
}
