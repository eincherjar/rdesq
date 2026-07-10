use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnEntry {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub protocol: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub group_id: Option<String>,
    pub tags: Vec<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub favorite: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnEntryInput {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub protocol: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub group_id: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub lang: String,
    pub theme: String,
    pub ui_scale: f64,
    pub start_with_system: bool,
    pub start_minimized: bool,
    pub close_to_tray: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub lang: Option<String>,
    pub theme: Option<String>,
    pub ui_scale: Option<f64>,
    pub start_with_system: Option<bool>,
    pub start_minimized: Option<bool>,
    pub close_to_tray: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagInfo {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PingTarget {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PingResult {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub reachable: bool,
    pub latency: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupOrder {
    pub id: String,
    pub sort_order: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportData {
    pub connections: Vec<ConnEntry>,
    pub groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub connections: Vec<ConnEntry>,
    pub groups: Vec<Group>,
}
