use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::engine::{compiler, export, hex, library, scanner};
use crate::models::{HexRegion, LibraryTree, ScanReport, ValidationResult};

#[tauri::command(async)]
pub fn validate_rules(source: String) -> ValidationResult {
    compiler::validate(&source)
}

pub struct LibraryRoot(pub std::path::PathBuf);

#[derive(Default)]
pub struct ScanRegistry(Mutex<HashMap<String, Arc<AtomicBool>>>);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanProgressEvent<'a> {
    scan_id: &'a str,
    scanned: usize,
    matched: usize,
    current_path: &'a str,
}

#[tauri::command(async)]
pub fn scan_paths(
    app: AppHandle,
    registry: State<'_, ScanRegistry>,
    root: State<'_, LibraryRoot>,
    source: String,
    library_rels: Vec<String>,
    paths: Vec<String>,
    scan_id: String,
) -> Result<ScanReport, String> {
    if paths.is_empty() {
        return Err("Nothing to scan".to_string());
    }

    let mut sources = Vec::new();
    if !source.trim().is_empty() {
        sources.push(("editor".to_string(), source));
    }
    for rel in &library_rels {
        sources.push((rel.clone(), library::read(&root.0, rel)?));
    }
    if sources.is_empty() {
        return Err("Write a rule or pick rules from the library first".to_string());
    }

    let rules = compiler::compile_set(&sources)?;

    let cancel = Arc::new(AtomicBool::new(false));
    registry
        .0
        .lock()
        .unwrap()
        .insert(scan_id.clone(), cancel.clone());

    let report = scanner::run_scan(&rules, &paths, &cancel, |progress| {
        let _ = app.emit(
            "scan-progress",
            ScanProgressEvent {
                scan_id: &scan_id,
                scanned: progress.scanned,
                matched: progress.matched,
                current_path: progress.current_path,
            },
        );
    });

    registry.0.lock().unwrap().remove(&scan_id);
    Ok(report)
}

#[tauri::command]
pub fn cancel_scan(registry: State<'_, ScanRegistry>, scan_id: String) {
    if let Some(flag) = registry.0.lock().unwrap().get(&scan_id) {
        flag.store(true, Ordering::Relaxed);
    }
}

#[tauri::command(async)]
pub fn export_report(report: ScanReport, format: String, path: String) -> Result<(), String> {
    export::export(&report, &format, &path)
}

#[tauri::command(async)]
pub fn read_hex_region(path: String, start: u64, length: usize) -> Result<HexRegion, String> {
    hex::read_region(std::path::Path::new(&path), start, length)
}

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
