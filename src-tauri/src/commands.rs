use tauri::State;

use crate::engine::{compiler, hex, library, scanner};
use crate::models::{HexRegion, LibraryTree, ScanReport, ValidationResult};

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

#[tauri::command(async)]
pub fn read_hex_region(path: String, start: u64, length: usize) -> Result<HexRegion, String> {
    hex::read_region(std::path::Path::new(&path), start, length)
}

pub struct LibraryRoot(pub std::path::PathBuf);

#[tauri::command(async)]
pub fn library_list(root: State<'_, LibraryRoot>) -> Result<LibraryTree, String> {
    library::list(&root.0)
}

#[tauri::command(async)]
pub fn library_save(
    root: State<'_, LibraryRoot>,
    collection: Option<String>,
    name: String,
    source: String,
) -> Result<String, String> {
    library::save(&root.0, collection.as_deref(), &name, &source)
}

#[tauri::command(async)]
pub fn library_read(root: State<'_, LibraryRoot>, rel: String) -> Result<String, String> {
    library::read(&root.0, &rel)
}

#[tauri::command(async)]
pub fn library_delete(root: State<'_, LibraryRoot>, rel: String) -> Result<(), String> {
    library::delete(&root.0, &rel)
}

#[tauri::command(async)]
pub fn library_create_collection(root: State<'_, LibraryRoot>, name: String) -> Result<(), String> {
    library::create_collection(&root.0, &name)
}

#[tauri::command(async)]
pub fn library_delete_collection(root: State<'_, LibraryRoot>, name: String) -> Result<(), String> {
    library::delete_collection(&root.0, &name)
}
