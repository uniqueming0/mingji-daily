use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AiConfigStore {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
}

impl Default for AiConfigStore {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            model: "deepseek-chat".into(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Category {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort: i64,
    pub status: i64,
    pub is_builtin: i64,
}

#[derive(Serialize, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub sort: i64,
    pub status: i64,
}

#[derive(Serialize, Clone)]
pub struct Bill {
    pub id: i64,
    pub r#type: i64,
    pub amount: i64,
    pub category_id: i64,
    pub account_id: i64,
    pub bill_date: String,
    pub remark: String,
    pub source: i64,
    pub status: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct BillFilter {
    pub r#type: Option<i64>,
    pub category_id: Option<i64>,
    pub account_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Deserialize)]
pub struct BillInput {
    pub r#type: i64,
    pub amount: i64,
    pub category_id: i64,
    pub account_id: i64,
    pub bill_date: String,
    pub remark: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct CycleRule {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub start_date: String,
    pub length: i64,
    pub length_unit: String,
    pub week_start: i64,
    pub month_start_day: i64,
    pub end_date: Option<String>,
    pub status: i64,
}

#[derive(Deserialize)]
pub struct CycleRuleInput {
    pub name: String,
    pub r#type: String,
    pub start_date: String,
    pub length: i64,
    pub length_unit: String,
    pub week_start: i64,
    pub month_start_day: i64,
    pub end_date: Option<String>,
    pub status: i64,
}

#[derive(Serialize)]
pub struct Bootstrap {
    pub categories: Vec<Category>,
    pub accounts: Vec<Account>,
    pub cycle_rules: Vec<CycleRule>,
}

#[derive(Serialize, Clone)]
pub struct Media {
    pub id: i64,
    pub bill_id: i64,
    pub r#type: i64,
    pub file_name: String,
    pub ext: String,
    pub size: i64,
    pub path: String,
    pub thumb_path: Option<String>,
}

#[derive(Serialize)]
pub struct MediaUsage {
    pub count: i64,
    pub bytes: i64,
}

#[derive(Serialize)]
pub struct ImportFile {
    pub name: String,
    pub ext: String,
    pub size: i64,
    pub text: Option<String>,
    pub base64: Option<String>,
    pub md5: String,
}

#[derive(Deserialize)]
pub struct ImportItemInput {
    pub r#type: i64,
    pub amount: i64,
    pub category_id: i64,
    pub bill_date: String,
    pub remark: String,
}

#[derive(Deserialize)]
pub struct ConfirmImportInput {
    pub file_name: String,
    pub file_md5: String,
    pub items: Vec<ImportItemInput>,
}

#[derive(Serialize, Clone)]
pub struct ImportTask {
    pub id: i64,
    pub file_name: String,
    pub file_md5: String,
    pub total: i64,
    pub created_at: String,
}

#[derive(Serialize, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size: i64,
    pub with_media: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct OrphanInfo {
    pub count: i64,
    pub bytes: i64,
}

#[derive(Serialize)]
pub struct ExportResult {
    pub count: i64,
    pub path: String,
}

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub name: String,
    pub os: String,
}

#[derive(Serialize)]
pub struct AiConfigPublic {
    pub enabled: bool,
    pub has_key: bool,
    pub model: String,
}

#[derive(Deserialize)]
pub struct AiConfigInput {
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AiParsedLine {
    pub line: i64,
    pub date: String,
    pub amount: f64,
    pub r#type: i64,
    pub category: String,
    pub remark: String,
}

#[derive(Deserialize)]
pub struct CycleRange {
    pub start: String,
    pub end: String,
}
