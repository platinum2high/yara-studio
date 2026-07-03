use std::path::{Path, PathBuf};

use crate::engine::{compiler, library};
use crate::models::{
    EntryTestReport, LibraryTestReport, SampleResult, TestKind, TestSample, TestSamples,
};

const TESTS_SUFFIX: &str = ".tests";
const MATCH_DIR: &str = "match";
const NO_MATCH_DIR: &str = "no-match";

fn tests_dir(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let entry_path = library::resolve(root, rel)?;
    let stem = entry_path
        .file_stem()
        .ok_or_else(|| "Invalid rule file name".to_string())?
        .to_string_lossy()
        .into_owned();
    Ok(entry_path.with_file_name(format!("{stem}{TESTS_SUFFIX}")))
}

fn kind_dir(kind: TestKind) -> &'static str {
    match kind {
        TestKind::Match => MATCH_DIR,
        TestKind::NoMatch => NO_MATCH_DIR,
    }
}

fn samples_in(dir: &Path) -> Vec<TestSample> {
    let mut samples: Vec<TestSample> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            Some(TestSample {
                file_name: name,
                size: e.metadata().map(|m| m.len()).unwrap_or(0),
            })
        })
        .collect();
    samples.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    samples
}

pub fn list_samples(root: &Path, rel: &str) -> Result<TestSamples, String> {
    let dir = tests_dir(root, rel)?;
    Ok(TestSamples {
        expect_match: samples_in(&dir.join(MATCH_DIR)),
        expect_no_match: samples_in(&dir.join(NO_MATCH_DIR)),
    })
}

pub fn add_sample(root: &Path, rel: &str, kind: TestKind, source_path: &str) -> Result<(), String> {
    let source = Path::new(source_path);
    if !source.is_file() {
        return Err("Sample must be a file".to_string());
    }
    let file_name = source
        .file_name()
        .ok_or_else(|| "Invalid sample file name".to_string())?;

    let dir = tests_dir(root, rel)?.join(kind_dir(kind));
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create tests dir: {e}"))?;
    std::fs::copy(source, dir.join(file_name)).map_err(|e| format!("Cannot copy sample: {e}"))?;
    Ok(())
}

pub fn remove_sample(
    root: &Path,
    rel: &str,
    kind: TestKind,
    file_name: &str,
) -> Result<(), String> {
    if file_name.contains('/') || file_name.contains('\\') || file_name.starts_with('.') {
        return Err("Invalid sample name".to_string());
    }
    let path = tests_dir(root, rel)?.join(kind_dir(kind)).join(file_name);
    std::fs::remove_file(&path).map_err(|e| format!("Cannot remove sample: {e}"))
}

fn run_entry(root: &Path, rel: &str) -> Result<EntryTestReport, String> {
    let source = library::read(root, rel)?;
    let samples = list_samples(root, rel)?;

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    let rules = match compiler::compile_set(&[(rel.to_string(), source)]) {
        Ok(rules) => Some(rules),
        Err(e) => {
            return Ok(EntryTestReport {
                rel: rel.to_string(),
                compile_error: Some(e),
                results,
                passed: 0,
                failed: samples.expect_match.len() + samples.expect_no_match.len(),
            });
        }
    };
    let rules = rules.unwrap();
    let dir = tests_dir(root, rel)?;

    let mut check = |kind: TestKind, sample: &TestSample| {
        let path = dir.join(kind_dir(kind)).join(&sample.file_name);
        let (matched_rules, error) = match std::fs::read(&path) {
            Ok(data) => {
                let mut scanner = yara_x::Scanner::new(&rules);
                match scanner.scan(&data) {
                    Ok(scan) => (
                        scan.matching_rules()
                            .map(|r| r.identifier().to_string())
                            .collect::<Vec<_>>(),
                        None,
                    ),
                    Err(e) => (Vec::new(), Some(format!("Scan failed: {e}"))),
                }
            }
            Err(e) => (Vec::new(), Some(format!("Cannot read sample: {e}"))),
        };

        let ok = error.is_none()
            && match kind {
                TestKind::Match => !matched_rules.is_empty(),
                TestKind::NoMatch => matched_rules.is_empty(),
            };
        if ok {
            passed += 1;
        } else {
            failed += 1;
        }
        results.push(SampleResult {
            file_name: sample.file_name.clone(),
            kind,
            passed: ok,
            matched_rules,
            error,
        });
    };

    for sample in &samples.expect_match {
        check(TestKind::Match, sample);
    }
    for sample in &samples.expect_no_match {
        check(TestKind::NoMatch, sample);
    }

    Ok(EntryTestReport {
        rel: rel.to_string(),
        compile_error: None,
        results,
        passed,
        failed,
    })
}

pub fn run(root: &Path, rels: &[String]) -> Result<LibraryTestReport, String> {
    let rels: Vec<String> = if rels.is_empty() {
        let tree = library::list(root)?;
        tree.entries
            .iter()
            .map(|e| e.rel.clone())
            .chain(
                tree.collections
                    .iter()
                    .flat_map(|c| c.entries.iter().map(|e| e.rel.clone())),
            )
            .collect()
    } else {
        rels.to_vec()
    };

    let mut entries = Vec::new();
    let mut skipped = 0;
    for rel in &rels {
        let report = run_entry(root, rel)?;
        if report.results.is_empty() && report.compile_error.is_none() {
            skipped += 1;
        } else {
            entries.push(report);
        }
    }

    Ok(LibraryTestReport {
        total_passed: entries.iter().map(|e| e.passed).sum(),
        total_failed: entries.iter().map(|e| e.failed).sum(),
        entries_without_tests: skipped,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULE: &str = r#"
rule TestTarget {
    strings:
        $a = "MALICIOUS_MARKER"
    condition:
        $a
}
"#;

    fn setup(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("yara-studio-rt-{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        library::save(&root, Some("c"), "target", RULE).unwrap();
        root
    }

    fn sample_file(tag: &str, name: &str, content: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("yara-studio-rt-samples-{tag}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn passing_and_failing_samples_are_judged_correctly() {
        let root = setup("judge");
        let hit = sample_file("judge", "hit.bin", b"xxMALICIOUS_MARKERxx");
        let clean = sample_file("judge", "clean.bin", b"nothing here");

        add_sample(
            &root,
            "c/target.yar",
            TestKind::Match,
            &hit.display().to_string(),
        )
        .unwrap();
        add_sample(
            &root,
            "c/target.yar",
            TestKind::Match,
            &clean.display().to_string(),
        )
        .unwrap();
        add_sample(
            &root,
            "c/target.yar",
            TestKind::NoMatch,
            &clean.display().to_string(),
        )
        .unwrap();

        let report = run(&root, &[]).unwrap();
        assert_eq!(report.total_passed, 2);
        assert_eq!(report.total_failed, 1);

        let entry = &report.entries[0];
        let failing = entry.results.iter().find(|r| !r.passed).unwrap();
        assert_eq!(failing.file_name, "clean.bin");
        assert!(matches!(failing.kind, TestKind::Match));

        for p in [hit, clean] {
            std::fs::remove_file(p).unwrap();
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn entries_without_tests_are_skipped_not_failed() {
        let root = setup("skip");
        let report = run(&root, &[]).unwrap();
        assert_eq!(report.entries_without_tests, 1);
        assert!(report.entries.is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn sample_lifecycle_and_listing() {
        let root = setup("life");
        let s = sample_file("life", "s.bin", b"MALICIOUS_MARKER");

        add_sample(
            &root,
            "c/target.yar",
            TestKind::Match,
            &s.display().to_string(),
        )
        .unwrap();
        let listed = list_samples(&root, "c/target.yar").unwrap();
        assert_eq!(listed.expect_match.len(), 1);
        assert_eq!(listed.expect_match[0].size, 16);

        remove_sample(&root, "c/target.yar", TestKind::Match, "s.bin").unwrap();
        assert!(list_samples(&root, "c/target.yar")
            .unwrap()
            .expect_match
            .is_empty());

        std::fs::remove_file(s).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn broken_rule_fails_its_tests_with_compile_error() {
        let root = setup("broken");
        library::save(&root, Some("c"), "bad", "rule Bad { cond }").unwrap();
        let s = sample_file("broken", "x.bin", b"data");
        add_sample(
            &root,
            "c/bad.yar",
            TestKind::Match,
            &s.display().to_string(),
        )
        .unwrap();

        let report = run(&root, &["c/bad.yar".to_string()]).unwrap();
        assert!(report.entries[0].compile_error.is_some());
        assert_eq!(report.total_failed, 1);

        std::fs::remove_file(s).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn tests_folder_never_appears_as_collection() {
        let root = setup("hidden");
        let s = sample_file("hidden", "h.bin", b"MALICIOUS_MARKER");
        // Root-level entry gets a sibling .tests folder at library root.
        library::save(&root, None, "rootrule", RULE).unwrap();
        add_sample(
            &root,
            "rootrule.yar",
            TestKind::Match,
            &s.display().to_string(),
        )
        .unwrap();

        let tree = library::list(&root).unwrap();
        assert!(tree
            .collections
            .iter()
            .all(|c| c.name != format!("rootrule{TESTS_SUFFIX}")));

        std::fs::remove_file(s).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
    }
}
