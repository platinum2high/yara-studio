use std::collections::HashSet;

use yara_x::{Compiler, Scanner, SourceCode};

const SUITE: &str = include_str!("fixtures/compat_suite.yar");

fn sample_data() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"MZ\x90\x00");
    data.extend_from_slice(b"STUDIO_MARKER_PLAIN ");
    data.extend_from_slice(b"sTuDiO_mArKeR_cAsE ");
    for ch in "STUDIO_MARKER_WIDE".bytes() {
        data.extend_from_slice(&[ch, 0]);
    }
    data.extend_from_slice(b" STUDIO_WORD ");
    data.extend_from_slice(b"\xDE\xAD\x77\xEF");
    data.extend_from_slice(b"\xCA\xFE\x01\x02\x03\xBA\xBE");
    data.extend_from_slice(b"\x00\xBB\xCC\x00");
    data.extend_from_slice(b"http://beacon.example.com ");
    data.extend_from_slice(b"VERSION_04.12 ");
    data.extend_from_slice(b"REPEATED_TOKEN..REPEATED_TOKEN..REPEATED_TOKEN");
    data
}

fn compile_suite() -> yara_x::Rules {
    let mut compiler = Compiler::new();
    compiler
        .add_source(SourceCode::from(SUITE).with_origin("compat_suite.yar"))
        .expect("compat suite must compile without errors");
    assert!(compiler.errors().is_empty());
    compiler.build()
}

#[test]
fn full_language_surface_compiles_cleanly() {
    compile_suite();
}

#[test]
fn scan_matches_every_suite_rule() {
    let rules = compile_suite();
    let mut scanner = Scanner::new(&rules);
    let data = sample_data();
    let results = scanner.scan(&data).expect("scan must succeed");

    let matched: HashSet<&str> = results.matching_rules().map(|r| r.identifier()).collect();

    for expected in [
        "TextStringModifiers",
        "HexPatterns",
        "RegexPatterns",
        "CountsAndOffsets",
        "ModuleFunctions",
        "FilesizeAndInts",
    ] {
        assert!(matched.contains(expected), "rule {expected} did not match");
    }
}

#[test]
fn match_offsets_and_data_are_exact() {
    let rules = compile_suite();
    let mut scanner = Scanner::new(&rules);
    let data = sample_data();
    let results = scanner.scan(&data).expect("scan must succeed");

    let rule = results
        .matching_rules()
        .find(|r| r.identifier() == "TextStringModifiers")
        .unwrap();
    let pattern = rule
        .patterns()
        .find(|p| p.identifier() == "$plain")
        .unwrap();
    let m = pattern.matches().next().unwrap();

    assert_eq!(m.range().start, 4);
    assert_eq!(m.data(), b"STUDIO_MARKER_PLAIN");
}

#[test]
fn compile_errors_carry_line_and_span() {
    let mut compiler = Compiler::new();
    let bad = "rule Broken {\n    strings:\n        $a = \"x\"\n    condition:\n        $a and\n}";
    let _ = compiler.add_source(bad);

    let errors = compiler.errors();
    assert!(!errors.is_empty());

    let label = errors[0].labels().next().expect("error must have a label");
    assert!(label.span().start() > 0);
}

#[test]
fn warnings_are_reported_for_sloppy_rules() {
    let mut compiler = Compiler::new();
    let sloppy =
        "rule Sloppy {\n    strings:\n        $a = \"ab\"\n    condition:\n        $a or true\n}";
    let _ = compiler.add_source(sloppy);

    assert!(compiler.errors().is_empty());
    assert!(!compiler.warnings().is_empty());
}
