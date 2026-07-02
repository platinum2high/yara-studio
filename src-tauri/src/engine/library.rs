use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::models::{LibraryCollection, LibraryEntry, LibraryTree};

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 {
        return Err("Name must be 1-128 characters".to_string());
    }
    let ok = name
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'));
    if !ok || name.starts_with('.') || name.contains("..") {
        return Err(format!("Invalid name: {name}"));
    }
    Ok(())
}

fn resolve(root: &Path, rel: &str) -> Result<PathBuf, String> {
    for part in Path::new(rel).components() {
        match part {
            std::path::Component::Normal(c) => {
                validate_name(&c.to_string_lossy())?;
            }
            _ => return Err(format!("Invalid path: {rel}")),
        }
    }
    Ok(root.join(rel))
}

struct ParsedSource {
    rule_names: Vec<String>,
    tags: Vec<String>,
    description: Option<String>,
    compiles: bool,
}

fn parse_source(source: &str) -> ParsedSource {
    let mut compiler = yara_x::Compiler::new();
    let compiles = compiler.add_source(source).is_ok() && compiler.errors().is_empty();

    if compiles {
        let rules = compiler.build();
        let mut rule_names = Vec::new();
        let mut tags: Vec<String> = Vec::new();
        let mut description = None;

        for rule in rules.iter() {
            rule_names.push(rule.identifier().to_string());
            for tag in rule.tags() {
                let t = tag.identifier().to_string();
                if !tags.contains(&t) {
                    tags.push(t);
                }
            }
            if description.is_none() {
                for (key, value) in rule.metadata() {
                    if key == "description" {
                        if let yara_x::MetaValue::String(s) = value {
                            description = Some(s.to_string());
                        }
                    }
                }
            }
        }
        return ParsedSource {
            rule_names,
            tags,
            description,
            compiles,
        };
    }

    // Broken sources still show up in the library; a cheap scan for rule
    // headers keeps them identifiable.
    let mut rule_names = Vec::new();
    for line in source.lines() {
        let line = line.trim_start();
        let rest = line
            .strip_prefix("private rule ")
            .or_else(|| line.strip_prefix("global rule "))
            .or_else(|| line.strip_prefix("rule "));
        if let Some(rest) = rest {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                rule_names.push(name);
            }
        }
    }
    ParsedSource {
        rule_names,
        tags: Vec::new(),
        description: None,
        compiles: false,
    }
}

fn entry_from_file(root: &Path, path: &Path) -> Option<LibraryEntry> {
    let source = std::fs::read_to_string(path).ok()?;
    let parsed = parse_source(&source);
    let rel = path.strip_prefix(root).ok()?.to_string_lossy().into_owned();
    let modified_epoch_ms = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    Some(LibraryEntry {
        rel,
        file_name: path.file_name()?.to_string_lossy().into_owned(),
        rule_names: parsed.rule_names,
        tags: parsed.tags,
        description: parsed.description,
        compiles: parsed.compiles,
        modified_epoch_ms,
    })
}

fn yar_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("yar") | Some("yara")
                )
        })
        .collect();
    files.sort();
    files
}

pub fn list(root: &Path) -> Result<LibraryTree, String> {
    std::fs::create_dir_all(root).map_err(|e| format!("Cannot create library: {e}"))?;

    let entries = yar_files(root)
        .iter()
        .filter_map(|p| entry_from_file(root, p))
        .collect();

    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .map_err(|e| format!("Cannot read library: {e}"))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    let collections = dirs
        .iter()
        .filter_map(|dir| {
            let name = dir.file_name()?.to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            Some(LibraryCollection {
                name,
                entries: yar_files(dir)
                    .iter()
                    .filter_map(|p| entry_from_file(root, p))
                    .collect(),
            })
        })
        .collect();

    Ok(LibraryTree {
        entries,
        collections,
    })
}

pub fn save(
    root: &Path,
    collection: Option<&str>,
    name: &str,
    source: &str,
) -> Result<String, String> {
    validate_name(name)?;
    let file_name = if name.ends_with(".yar") || name.ends_with(".yara") {
        name.to_string()
    } else {
        format!("{name}.yar")
    };

    let dir = match collection {
        Some(c) => {
            validate_name(c)?;
            root.join(c)
        }
        None => root.to_path_buf(),
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create collection: {e}"))?;

    let path = dir.join(&file_name);
    std::fs::write(&path, source).map_err(|e| format!("Cannot save rule: {e}"))?;

    Ok(path
        .strip_prefix(root)
        .map_err(|_| "Path escaped library root".to_string())?
        .to_string_lossy()
        .into_owned())
}

pub fn read(root: &Path, rel: &str) -> Result<String, String> {
    let path = resolve(root, rel)?;
    std::fs::read_to_string(&path).map_err(|e| format!("Cannot read rule: {e}"))
}

pub fn delete(root: &Path, rel: &str) -> Result<(), String> {
    let path = resolve(root, rel)?;
    if !path.is_file() {
        return Err("Not a rule file".to_string());
    }
    std::fs::remove_file(&path).map_err(|e| format!("Cannot delete rule: {e}"))
}

pub fn create_collection(root: &Path, name: &str) -> Result<(), String> {
    validate_name(name)?;
    std::fs::create_dir_all(root.join(name)).map_err(|e| format!("Cannot create: {e}"))
}

pub fn delete_collection(root: &Path, name: &str) -> Result<(), String> {
    validate_name(name)?;
    let path = root.join(name);
    if !path.is_dir() {
        return Err("Not a collection".to_string());
    }
    std::fs::remove_dir_all(&path).map_err(|e| format!("Cannot delete collection: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULE: &str = r#"
rule LibDemo : trojan win32 {
    meta:
        description = "Library demo rule"
    strings:
        $a = "x"
    condition:
        $a
}
"#;

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("yara-studio-lib-{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn save_list_read_delete_roundtrip() {
        let root = temp_root("roundtrip");

        let rel = save(&root, Some("apt"), "demo", RULE).unwrap();
        assert_eq!(rel, "apt/demo.yar");

        let tree = list(&root).unwrap();
        assert_eq!(tree.collections.len(), 1);
        let entry = &tree.collections[0].entries[0];
        assert_eq!(entry.rule_names, vec!["LibDemo"]);
        assert_eq!(entry.tags, vec!["trojan", "win32"]);
        assert_eq!(entry.description.as_deref(), Some("Library demo rule"));
        assert!(entry.compiles);

        assert_eq!(read(&root, &rel).unwrap(), RULE);

        delete(&root, &rel).unwrap();
        assert!(list(&root).unwrap().collections[0].entries.is_empty());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn broken_rules_are_listed_but_flagged() {
        let root = temp_root("broken");
        save(&root, None, "broken", "rule Oops { cond }").unwrap();

        let tree = list(&root).unwrap();
        let entry = &tree.entries[0];
        assert!(!entry.compiles);
        assert_eq!(entry.rule_names, vec!["Oops"]);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn path_traversal_is_rejected() {
        let root = temp_root("traversal");
        std::fs::create_dir_all(&root).unwrap();

        assert!(read(&root, "../outside.yar").is_err());
        assert!(read(&root, "/etc/passwd").is_err());
        assert!(save(&root, Some("../evil"), "x", RULE).is_err());
        assert!(delete(&root, "a/../../b.yar").is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn collection_lifecycle() {
        let root = temp_root("collections");

        create_collection(&root, "malware").unwrap();
        assert_eq!(list(&root).unwrap().collections[0].name, "malware");

        save(&root, Some("malware"), "a", RULE).unwrap();
        delete_collection(&root, "malware").unwrap();
        assert!(list(&root).unwrap().collections.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }
}
