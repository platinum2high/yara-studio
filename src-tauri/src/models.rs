use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub title: String,
    pub detail: String,
    pub line: usize,
    pub column: usize,
    pub from: usize,
    pub to: usize,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub ok: bool,
    pub rule_count: usize,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StringMatch {
    pub identifier: String,
    pub offset: usize,
    pub length: usize,
    pub matched_hex: String,
    pub matched_ascii: String,
    pub context_before_hex: String,
    pub context_after_hex: String,
    pub xor_key: Option<u8>,
    pub truncated: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RuleMatch {
    pub identifier: String,
    pub namespace: String,
    pub tags: Vec<String>,
    pub meta: serde_json::Value,
    pub string_matches: Vec<StringMatch>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FileStatus {
    #[serde(rename = "matched")]
    Matched,
    #[serde(rename = "clean")]
    Clean,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    pub path: String,
    pub file_name: String,
    pub size: u64,
    pub sha256: Option<String>,
    pub duration_ms: u64,
    pub status: FileStatus,
    pub error: Option<String>,
    pub rule_matches: Vec<RuleMatch>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub started_at_epoch_ms: u64,
    pub duration_ms: u64,
    pub scanned_files: usize,
    pub matched_files: usize,
    pub error_files: usize,
    pub clean_files: usize,
    pub rule_count: usize,
    pub cancelled: bool,
    pub truncated: bool,
    pub results: Vec<FileResult>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HexRegion {
    pub file_size: u64,
    pub start: u64,
    pub bytes_hex: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub rel: String,
    pub file_name: String,
    pub rule_names: Vec<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub compiles: bool,
    pub modified_epoch_ms: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCollection {
    pub name: String,
    pub entries: Vec<LibraryEntry>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTree {
    pub entries: Vec<LibraryEntry>,
    pub collections: Vec<LibraryCollection>,
}
