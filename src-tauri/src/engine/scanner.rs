use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use yara_x::{Rules, Scanner};

use crate::models::{FileResult, FileStatus, RuleMatch, ScanReport, StringMatch};

const MATCH_CONTEXT_BYTES: usize = 16;
const MAX_MATCH_PREVIEW_BYTES: usize = 64;
const MAX_MATCHES_PER_PATTERN: usize = 50;
const SCAN_TIMEOUT: Duration = Duration::from_secs(30);

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn ascii_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7F).contains(&b) {
                b as char
            } else {
                '·'
            }
        })
        .collect()
}

fn string_matches(pattern: &yara_x::Pattern) -> Vec<StringMatch> {
    pattern
        .matches()
        .take(MAX_MATCHES_PER_PATTERN)
        .map(|m| {
            let range = m.range();
            let (context, match_range) = m.data_with_context();
            let before = &context[..match_range.start];
            let after = &context[match_range.end..];
            let matched = &context[match_range.clone()];
            let truncated = matched.len() > MAX_MATCH_PREVIEW_BYTES;
            let preview = &matched[..matched.len().min(MAX_MATCH_PREVIEW_BYTES)];

            StringMatch {
                identifier: pattern.identifier().to_string(),
                offset: range.start,
                length: range.len(),
                matched_hex: hex(preview),
                matched_ascii: ascii_preview(preview),
                context_before_hex: hex(before),
                context_after_hex: hex(after),
                xor_key: m.xor_key(),
                truncated,
            }
        })
        .collect()
}

fn scan_one(rules: &Rules, path: &Path) -> FileResult {
    let started = Instant::now();
    let display_path = path.display().to_string();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| display_path.clone());

    let error = |size: u64, message: String| FileResult {
        path: display_path.clone(),
        file_name: file_name.clone(),
        size,
        sha256: None,
        duration_ms: started.elapsed().as_millis() as u64,
        status: FileStatus::Error,
        error: Some(message),
        rule_matches: Vec::new(),
    };

    if path.is_dir() {
        return error(
            0,
            "This is a directory — drop individual files to scan them".to_string(),
        );
    }

    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) => return error(0, format!("Cannot read file: {e}")),
    };

    let sha256: String = Sha256::digest(&data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let mut scanner = Scanner::new(rules);
    scanner
        .set_timeout(SCAN_TIMEOUT)
        .match_context_size(MATCH_CONTEXT_BYTES);

    let results = match scanner.scan(&data) {
        Ok(results) => results,
        Err(e) => return error(data.len() as u64, format!("Scan failed: {e}")),
    };

    let rule_matches: Vec<RuleMatch> = results
        .matching_rules()
        .map(|rule| RuleMatch {
            identifier: rule.identifier().to_string(),
            namespace: rule.namespace().to_string(),
            tags: rule.tags().map(|t| t.identifier().to_string()).collect(),
            meta: rule.metadata().into_json(),
            string_matches: rule.patterns().flat_map(|p| string_matches(&p)).collect(),
        })
        .collect();

    FileResult {
        path: display_path,
        file_name,
        size: data.len() as u64,
        sha256: Some(sha256),
        duration_ms: started.elapsed().as_millis() as u64,
        status: if rule_matches.is_empty() {
            FileStatus::Clean
        } else {
            FileStatus::Matched
        },
        error: None,
        rule_matches,
    }
}

pub fn scan_files(rules: &Rules, paths: &[String]) -> ScanReport {
    let started_at_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let results: Vec<FileResult> = paths
        .iter()
        .map(|p| scan_one(rules, Path::new(p)))
        .collect();

    ScanReport {
        started_at_epoch_ms,
        total_files: results.len(),
        matched_files: results
            .iter()
            .filter(|r| matches!(r.status, FileStatus::Matched))
            .count(),
        error_files: results
            .iter()
            .filter(|r| matches!(r.status, FileStatus::Error))
            .count(),
        rule_count: rules.iter().count(),
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::compiler::compile;

    const RULE: &str = r#"
rule FindMarker {
    meta:
        author = "test"
    strings:
        $m = "NEEDLE_IN_HAYSTACK"
    condition:
        $m
}
"#;

    fn temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("yara-studio-test-{name}"));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn match_offset_bytes_and_context_are_exact() {
        let rules = compile(RULE).unwrap();
        let path = temp_file("match.bin", b"prefix--NEEDLE_IN_HAYSTACK--suffix");

        let report = scan_files(&rules, &[path.display().to_string()]);
        let file = &report.results[0];

        assert!(matches!(file.status, FileStatus::Matched));
        assert_eq!(file.rule_matches.len(), 1);

        let m = &file.rule_matches[0].string_matches[0];
        assert_eq!(m.offset, 8);
        assert_eq!(m.length, 18);
        assert_eq!(m.matched_ascii, "NEEDLE_IN_HAYSTACK");
        assert_eq!(m.matched_hex.split(' ').count(), 18);
        assert_eq!(m.context_before_hex, hex(b"prefix--"));
        assert_eq!(m.context_after_hex, hex(b"--suffix"));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn clean_file_still_reports_hash_and_size() {
        let rules = compile(RULE).unwrap();
        let path = temp_file("clean.bin", b"nothing to see here");

        let report = scan_files(&rules, &[path.display().to_string()]);
        let file = &report.results[0];

        assert!(matches!(file.status, FileStatus::Clean));
        assert_eq!(file.size, 19);
        assert_eq!(file.sha256.as_ref().unwrap().len(), 64);
        assert_eq!(report.matched_files, 0);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unreadable_path_becomes_file_error_not_panic() {
        let rules = compile(RULE).unwrap();
        let report = scan_files(&rules, &["/definitely/not/a/real/path".to_string()]);

        assert!(matches!(report.results[0].status, FileStatus::Error));
        assert_eq!(report.error_files, 1);
    }

    #[test]
    fn directory_is_rejected_with_clear_message() {
        let rules = compile(RULE).unwrap();
        let dir = std::env::temp_dir();
        let report = scan_files(&rules, &[dir.display().to_string()]);

        let file = &report.results[0];
        assert!(matches!(file.status, FileStatus::Error));
        assert!(file.error.as_ref().unwrap().contains("directory"));
    }

    #[test]
    fn ascii_preview_masks_unprintable_bytes() {
        assert_eq!(ascii_preview(b"AB\x00\xFFcd"), "AB··cd");
    }
}
