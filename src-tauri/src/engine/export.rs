use std::fmt::Write as _;

use crate::models::{FileStatus, ScanReport};

pub fn to_json(report: &ScanReport) -> Result<String, String> {
    serde_json::to_string_pretty(report).map_err(|e| format!("Cannot serialize report: {e}"))
}

fn csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn status_str(status: &FileStatus) -> &'static str {
    match status {
        FileStatus::Matched => "matched",
        FileStatus::Clean => "clean",
        FileStatus::Error => "error",
    }
}

pub fn to_csv(report: &ScanReport) -> String {
    let mut out = String::from(
        "file,sha256,size,status,rule,namespace,tags,string,offset,length,xor_key,matched_ascii,error\n",
    );

    for file in &report.results {
        let base = [
            csv_field(&file.path),
            file.sha256.clone().unwrap_or_default(),
            file.size.to_string(),
            status_str(&file.status).to_string(),
        ]
        .join(",");

        let mut wrote_row = false;
        for rule in &file.rule_matches {
            let rule_part = [
                csv_field(&rule.identifier),
                csv_field(&rule.namespace),
                csv_field(&rule.tags.join(" ")),
            ]
            .join(",");

            if rule.string_matches.is_empty() {
                let _ = writeln!(out, "{base},{rule_part},,,,,");
                wrote_row = true;
            }
            for m in &rule.string_matches {
                let xor = m.xor_key.map(|k| format!("0x{k:02x}")).unwrap_or_default();
                let _ = writeln!(
                    out,
                    "{base},{rule_part},{},{},{},{},{},",
                    csv_field(&m.identifier),
                    format_args!("0x{:x}", m.offset),
                    m.length,
                    xor,
                    csv_field(&m.matched_ascii),
                );
                wrote_row = true;
            }
        }
        if !wrote_row {
            let _ = writeln!(
                out,
                "{base},,,,,,,,{}",
                csv_field(file.error.as_deref().unwrap_or_default())
            );
        }
    }
    out
}

pub fn to_txt(report: &ScanReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "YARA Studio scan report");
    let _ = writeln!(out, "=======================");
    let _ = writeln!(
        out,
        "files scanned : {} ({} matched, {} clean, {} errors)",
        report.scanned_files, report.matched_files, report.clean_files, report.error_files
    );
    let _ = writeln!(out, "rules loaded  : {}", report.rule_count);
    let _ = writeln!(out, "duration      : {} ms", report.duration_ms);
    if report.cancelled {
        let _ = writeln!(out, "NOTE: scan was cancelled — results are partial");
    }
    if report.truncated {
        let _ = writeln!(out, "NOTE: listing truncated");
    }

    for file in &report.results {
        let _ = writeln!(out, "\n[{}] {}", status_str(&file.status), file.path);
        if let Some(sha) = &file.sha256 {
            let _ = writeln!(out, "  sha256 : {sha}");
        }
        let _ = writeln!(out, "  size   : {} bytes", file.size);
        if let Some(error) = &file.error {
            let _ = writeln!(out, "  error  : {error}");
        }
        for rule in &file.rule_matches {
            let tags = if rule.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", rule.tags.join(", "))
            };
            let _ = writeln!(
                out,
                "  rule {} ({}){}",
                rule.identifier, rule.namespace, tags
            );
            for m in &rule.string_matches {
                let xor = m
                    .xor_key
                    .map(|k| format!(" xor=0x{k:02x}"))
                    .unwrap_or_default();
                let _ = writeln!(
                    out,
                    "    {} @ 0x{:08X} len={}{}  {}",
                    m.identifier, m.offset, m.length, xor, m.matched_ascii
                );
            }
        }
    }
    out
}

pub fn export(report: &ScanReport, format: &str, path: &str) -> Result<(), String> {
    let contents = match format {
        "json" => to_json(report)?,
        "csv" => to_csv(report),
        "txt" => to_txt(report),
        other => return Err(format!("Unknown export format: {other}")),
    };
    std::fs::write(path, contents).map_err(|e| format!("Cannot write report: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileResult, RuleMatch, StringMatch};

    fn sample() -> ScanReport {
        ScanReport {
            started_at_epoch_ms: 0,
            duration_ms: 42,
            scanned_files: 2,
            matched_files: 1,
            error_files: 1,
            clean_files: 0,
            rule_count: 1,
            cancelled: false,
            truncated: false,
            results: vec![
                FileResult {
                    path: "/tmp/evil, \"quoted\".bin".to_string(),
                    file_name: "evil.bin".to_string(),
                    size: 100,
                    sha256: Some("ab".repeat(32)),
                    duration_ms: 5,
                    status: FileStatus::Matched,
                    error: None,
                    rule_matches: vec![RuleMatch {
                        identifier: "Demo".to_string(),
                        namespace: "editor".to_string(),
                        tags: vec!["apt".to_string()],
                        meta: serde_json::Value::Null,
                        string_matches: vec![StringMatch {
                            identifier: "$a".to_string(),
                            offset: 16,
                            length: 4,
                            matched_hex: "41 42 43 44".to_string(),
                            matched_ascii: "ABCD".to_string(),
                            context_before_hex: String::new(),
                            context_after_hex: String::new(),
                            xor_key: Some(0x5a),
                            truncated: false,
                        }],
                    }],
                },
                FileResult {
                    path: "/tmp/gone.bin".to_string(),
                    file_name: "gone.bin".to_string(),
                    size: 0,
                    sha256: None,
                    duration_ms: 0,
                    status: FileStatus::Error,
                    error: Some("Cannot read file".to_string()),
                    rule_matches: vec![],
                },
            ],
        }
    }

    #[test]
    fn json_roundtrips() {
        let json = to_json(&sample()).unwrap();
        let back: ScanReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.results.len(), 2);
        assert_eq!(back.results[0].rule_matches[0].identifier, "Demo");
    }

    #[test]
    fn csv_escapes_and_lists_matches_and_errors() {
        let csv = to_csv(&sample());
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("file,sha256,size,status,rule"));
        assert!(lines[1].contains("\"/tmp/evil, \"\"quoted\"\".bin\""));
        assert!(lines[1].contains("$a,0x10,4,0x5a,ABCD"));
        assert!(lines[2].contains("error"));
        assert!(lines[2].contains("Cannot read file"));
    }

    #[test]
    fn txt_contains_summary_and_match_details() {
        let txt = to_txt(&sample());
        assert!(txt.contains("2 (1 matched, 0 clean, 1 errors)"));
        assert!(txt.contains("rule Demo (editor) [apt]"));
        assert!(txt.contains("$a @ 0x00000010 len=4 xor=0x5a  ABCD"));
        assert!(txt.contains("[error] /tmp/gone.bin"));
    }

    #[test]
    fn unknown_format_is_rejected() {
        assert!(export(&sample(), "xml", "/tmp/x").is_err());
    }
}
