use crate::engine::{compiler, scanner};
use crate::models::{ScanReport, ValidationResult};

#[tauri::command(async)]
pub fn validate_rules(source: String) -> ValidationResult {
    compiler::validate(&source)
}

#[tauri::command(async)]
pub fn scan_paths(source: String, paths: Vec<String>) -> Result<ScanReport, String> {
    if source.trim().is_empty() {
        return Err("Write a rule before scanning".to_string());
    }
    if paths.is_empty() {
        return Err("Nothing to scan".to_string());
    }

    let rules = compiler::compile(&source).map_err(|errors| {
        format!(
            "Rules do not compile ({} error{}) — fix them before scanning",
            errors.len(),
            if errors.len() == 1 { "" } else { "s" }
        )
    })?;

    Ok(scanner::scan_files(&rules, &paths))
}
