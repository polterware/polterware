//! The deploy targets of a Structure component, per environment.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// One environment link for a Structure node. `provider` is a catalog id and
/// `resource` names the remote target. Credentials are never stored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployment {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub environment_id: String,
    /// The environment name, for display. The table stores the id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub environment: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub resource: String,
    /// The platform's stable id for the resource, empty until linked.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_id: String,
    /// The resource's page on the platform's dashboard, filled by the core.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// The latest deploy the platform's CLI reported; `None` when never checked.
    #[serde(skip)]
    pub check: Option<DeployCheck>,
}

/// The latest deploy of a link as its platform reported it. `state` is ok,
/// building, failed, stopped, or unknown.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployCheck {
    pub state: String,
    pub summary: String,
    pub url: String,
    pub checked_at: String,
}

/// Reads a deploys field from a planned op or a replayed `change_ops` JSON
/// value. `None` is returned when the value is not a deployment list; JSON null
/// is an empty list.
pub fn deployments_from(value: &Value) -> Option<Vec<Deployment>> {
    match value {
        Value::Null => Some(Vec::new()),
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let fields = item.as_object()?;
                out.push(Deployment {
                    id: field_string(fields, "id"),
                    environment_id: field_string(fields, "environment_id"),
                    environment: field_string(fields, "environment"),
                    provider: field_string(fields, "provider"),
                    resource: field_string(fields, "resource"),
                    target_id: field_string(fields, "target_id"),
                    url: String::new(),
                    check: None,
                });
            }
            Some(out)
        }
        _ => None,
    }
}

fn field_string(fields: &Map<String, Value>, key: &str) -> String {
    match fields.get(key) {
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    }
}
