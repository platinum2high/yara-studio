use std::collections::HashMap;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::models::{CandidateString, SampleAnalysis, StringCategory, StringKind};

const MIN_STRING_LEN: usize = 6;
const MAX_STRING_LEN: usize = 120;
const MAX_CANDIDATES: usize = 200;
const MAX_SAMPLE_BYTES: u64 = 256 * 1024 * 1024;

fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let len = data.len() as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn detect_file_type(data: &[u8]) -> Option<String> {
    let sigs: &[(&[u8], &str)] = &[
        (b"MZ", "PE executable (MZ)"),
        (b"\x7fELF", "ELF executable"),
        (b"\xfe\xed\xfa\xce", "Mach-O (32-bit)"),
        (b"\xfe\xed\xfa\xcf", "Mach-O (64-bit)"),
        (b"\xce\xfa\xed\xfe", "Mach-O (32-bit, LE)"),
        (b"\xcf\xfa\xed\xfe", "Mach-O (64-bit, LE)"),
        (b"\xca\xfe\xba\xbe", "Mach-O universal binary"),
        (b"\xbe\xba\xfe\xca", "Mach-O universal binary (LE)"),
        (b"PK\x03\x04", "ZIP archive (also docx/jar/apk)"),
        (b"%PDF", "PDF document"),
        (b"\x1f\x8b", "gzip archive"),
        (b"Rar!", "RAR archive"),
        (b"7z\xbc\xaf", "7-Zip archive"),
        (b"\xd0\xcf\x11\xe0", "OLE2 document (doc/xls/msi)"),
        (b"#!", "script with shebang"),
        (b"\x4c\x00\x00\x00", "Windows LNK"),
        (b"dex\n", "Android DEX"),
        (b"SQLite format 3", "SQLite database"),
    ];
    sigs.iter()
        .find(|(magic, _)| data.starts_with(magic))
        .map(|(_, name)| name.to_string())
}

fn is_printable(b: u8) -> bool {
    (0x20..0x7f).contains(&b)
}

fn categorize(value: &str) -> StringCategory {
    let lower = value.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("ftp://")
    {
        return StringCategory::Url;
    }
    if value.split('.').count() == 4
        && value
            .split('.')
            .all(|p| !p.is_empty() && p.len() <= 3 && p.bytes().all(|b| b.is_ascii_digit()))
    {
        return StringCategory::Ip;
    }
    if lower.contains(".pdb") {
        return StringCategory::Pdb;
    }
    if lower.starts_with("hkey_") || lower.contains("\\software\\") || lower.contains("\\run\\") {
        return StringCategory::Registry;
    }
    if lower.contains("mozilla/") || lower.contains("user-agent") {
        return StringCategory::UserAgent;
    }
    if value.contains('@') && value.contains('.') && !value.contains(' ') {
        return StringCategory::Email;
    }
    if value.contains("\\") || value.contains("/") {
        let path_like = lower.ends_with(".exe")
            || lower.ends_with(".dll")
            || lower.ends_with(".bat")
            || lower.ends_with(".ps1")
            || lower.ends_with(".sh")
            || lower.starts_with("c:\\")
            || lower.starts_with("/tmp/")
            || lower.starts_with("/etc/")
            || lower.starts_with("%appdata%")
            || lower.starts_with("%temp%");
        if path_like {
            return StringCategory::Path;
        }
    }
    StringCategory::Plain
}

fn category_score(category: StringCategory) -> u32 {
    match category {
        StringCategory::Url => 100,
        StringCategory::Ip => 90,
        StringCategory::Pdb => 85,
        StringCategory::Registry => 80,
        StringCategory::UserAgent => 75,
        StringCategory::Email => 70,
        StringCategory::Path => 60,
        StringCategory::Plain => 10,
    }
}

struct Extracted {
    offset: usize,
    count: usize,
    kind: StringKind,
}

fn extract_strings(data: &[u8]) -> Vec<CandidateString> {
    let mut seen: HashMap<(String, StringKind), Extracted> = HashMap::new();

    let mut push = |value: &str, offset: usize, kind: StringKind| {
        let entry = seen.entry((value.to_string(), kind)).or_insert(Extracted {
            offset,
            count: 0,
            kind,
        });
        entry.count += 1;
    };

    // ASCII runs
    let mut start = None;
    for (i, &b) in data.iter().enumerate() {
        if is_printable(b) {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            if i - s >= MIN_STRING_LEN {
                let run = &data[s..(s + (i - s).min(MAX_STRING_LEN))];
                push(std::str::from_utf8(run).unwrap(), s, StringKind::Ascii);
            }
        }
    }
    if let Some(s) = start {
        if data.len() - s >= MIN_STRING_LEN {
            let run = &data[s..(s + (data.len() - s).min(MAX_STRING_LEN))];
            push(std::str::from_utf8(run).unwrap(), s, StringKind::Ascii);
        }
    }

    // UTF-16LE runs (printable byte followed by NUL)
    let mut wide: Vec<u8> = Vec::new();
    let mut wide_start = 0usize;
    let mut i = 0;
    while i + 1 < data.len() {
        if is_printable(data[i]) && data[i + 1] == 0 {
            if wide.is_empty() {
                wide_start = i;
            }
            wide.push(data[i]);
            i += 2;
        } else {
            if wide.len() >= MIN_STRING_LEN {
                let text: String = wide
                    .iter()
                    .take(MAX_STRING_LEN)
                    .map(|&b| b as char)
                    .collect();
                push(&text, wide_start, StringKind::Wide);
            }
            wide.clear();
            i += 1;
        }
    }
    if wide.len() >= MIN_STRING_LEN {
        let text: String = wide
            .iter()
            .take(MAX_STRING_LEN)
            .map(|&b| b as char)
            .collect();
        push(&text, wide_start, StringKind::Wide);
    }

    let mut candidates: Vec<CandidateString> = seen
        .into_iter()
        .map(|((value, _), info)| {
            let category = categorize(&value);
            CandidateString {
                score: category_score(category) + (value.len().min(60) as u32) / 4,
                value,
                kind: info.kind,
                offset: info.offset,
                count: info.count,
                category,
            }
        })
        .collect();

    candidates.sort_by(|a, b| b.score.cmp(&a.score).then(a.offset.cmp(&b.offset)));
    candidates.truncate(MAX_CANDIDATES);
    candidates
}

pub fn analyze(path: &Path) -> Result<SampleAnalysis, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot access file: {e}"))?;
    if metadata.is_dir() {
        return Err("Pick a file, not a directory".to_string());
    }
    if metadata.len() > MAX_SAMPLE_BYTES {
        return Err(format!(
            "File exceeds the {} MB wizard limit",
            MAX_SAMPLE_BYTES / 1024 / 1024
        ));
    }

    let data = std::fs::read(path).map_err(|e| format!("Cannot read file: {e}"))?;
    let sha256: String = Sha256::digest(&data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let header_len = data.len().min(16);
    let header_hex = data[..header_len]
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(SampleAnalysis {
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        size: data.len() as u64,
        sha256,
        entropy: shannon_entropy(&data),
        file_type: detect_file_type(&data),
        header_hex,
        strings: extract_strings(&data),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_sample(name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("yara-studio-wiz-{name}"));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn extracts_and_ranks_ascii_and_wide_strings() {
        let mut data = Vec::new();
        data.extend_from_slice(b"MZ\x90\x00\x01\x02");
        data.extend_from_slice(b"http://evil-c2.example.com/gate.php\x00");
        data.extend_from_slice(b"\x01\x02plain_marker_string\x03");
        for ch in "WideRegistryKey".bytes() {
            data.extend_from_slice(&[ch, 0]);
        }
        data.extend_from_slice(b"\xff\xfe");
        data.extend_from_slice(b"repeat_me\x00\x01repeat_me\x00");

        let path = temp_sample("rank.bin", &data);
        let analysis = analyze(&path).unwrap();

        assert_eq!(analysis.file_type.as_deref(), Some("PE executable (MZ)"));
        assert!(analysis.header_hex.starts_with("4D 5A 90 00"));
        assert_eq!(analysis.sha256.len(), 64);

        let url = &analysis.strings[0];
        assert_eq!(url.value, "http://evil-c2.example.com/gate.php");
        assert!(matches!(url.category, StringCategory::Url));
        assert!(matches!(url.kind, StringKind::Ascii));

        let wide = analysis
            .strings
            .iter()
            .find(|s| s.value == "WideRegistryKey")
            .unwrap();
        assert!(matches!(wide.kind, StringKind::Wide));

        let repeated = analysis
            .strings
            .iter()
            .find(|s| s.value == "repeat_me")
            .unwrap();
        assert_eq!(repeated.count, 2);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn detects_common_magic_including_fat_macho() {
        assert_eq!(
            detect_file_type(b"\xca\xfe\xba\xbe\x00\x00").as_deref(),
            Some("Mach-O universal binary")
        );
        assert_eq!(
            detect_file_type(b"\xcf\xfa\xed\xfe...").as_deref(),
            Some("Mach-O (64-bit, LE)")
        );
        assert_eq!(
            detect_file_type(b"MZ\x90\x00").as_deref(),
            Some("PE executable (MZ)")
        );
        assert_eq!(
            detect_file_type(b"\x7fELF\x02").as_deref(),
            Some("ELF executable")
        );
        assert!(detect_file_type(b"random bytes").is_none());
    }

    #[test]
    fn categorizes_iocs() {
        assert!(matches!(
            categorize("https://x.example.org/a"),
            StringCategory::Url
        ));
        assert!(matches!(categorize("192.168.10.55"), StringCategory::Ip));
        assert!(matches!(
            categorize("C:\\dev\\payload\\Release\\bot.pdb"),
            StringCategory::Pdb
        ));
        assert!(matches!(
            categorize("HKEY_CURRENT_USER\\Software\\Microsoft"),
            StringCategory::Registry
        ));
        assert!(matches!(
            categorize("C:\\Windows\\payload.exe"),
            StringCategory::Path
        ));
        assert!(matches!(
            categorize("just some words"),
            StringCategory::Plain
        ));
        assert!(matches!(categorize("1.2.3.4.5"), StringCategory::Plain));
    }

    #[test]
    fn entropy_is_sane() {
        assert_eq!(shannon_entropy(&[0u8; 100]), 0.0);
        let all: Vec<u8> = (0..=255).collect();
        assert!(shannon_entropy(&all) > 7.9);
    }

    #[test]
    fn missing_file_and_directory_are_errors() {
        assert!(analyze(Path::new("/no/such/sample")).is_err());
        assert!(analyze(&std::env::temp_dir()).is_err());
    }
}
