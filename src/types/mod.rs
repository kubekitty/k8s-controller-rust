use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub resource: Option<String>,
    pub namespace: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub category: String,
    pub severity: String,
    pub condition: String,
}