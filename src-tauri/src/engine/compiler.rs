use yara_x::{Compiler, Rules, SourceCode};

use crate::models::{Diagnostic, DiagnosticLevel, ValidationResult};

// CodeMirror addresses documents in UTF-16 code units while yara-x spans
// are byte offsets, so every span has to be remapped before it reaches
// the frontend.
struct Location {
    line: usize,
    column: usize,
    utf16_offset: usize,
}

fn locate(source: &str, byte_offset: usize) -> Location {
    let mut line = 1;
    let mut column = 1;
    let mut utf16_offset = 0;
    let mut bytes_seen = 0;

    for ch in source.chars() {
        if bytes_seen >= byte_offset {
            break;
        }
        bytes_seen += ch.len_utf8();
        utf16_offset += ch.len_utf16();
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    Location {
        line,
        column,
        utf16_offset,
    }
}

fn diagnostic(
    source: &str,
    level: DiagnosticLevel,
    code: &str,
    title: &str,
    label: Option<(usize, usize, String)>,
) -> Diagnostic {
    let (span_start, span_end, detail) = label.unwrap_or((0, 0, String::new()));
    let start = locate(source, span_start);
    let end = locate(source, span_end);

    Diagnostic {
        level,
        code: code.to_string(),
        title: title.to_string(),
        detail,
        line: start.line,
        column: start.column,
        from: start.utf16_offset,
        to: end.utf16_offset,
    }
}

fn run_compiler(source: &str) -> (Compiler<'static>, Vec<Diagnostic>, Vec<Diagnostic>) {
    let mut compiler = Compiler::new();
    let _ = compiler.add_source(SourceCode::from(source).with_origin("editor"));

    let errors = compiler
        .errors()
        .iter()
        .map(|e| {
            let label = e
                .labels()
                .next()
                .map(|l| (l.span().start(), l.span().end(), l.text().to_string()));
            diagnostic(source, DiagnosticLevel::Error, e.code(), e.title(), label)
        })
        .collect();

    let warnings = compiler
        .warnings()
        .iter()
        .map(|w| {
            let label = w
                .labels()
                .next()
                .map(|l| (l.span().start(), l.span().end(), l.text().to_string()));
            diagnostic(source, DiagnosticLevel::Warning, w.code(), w.title(), label)
        })
        .collect();

    (compiler, errors, warnings)
}

pub fn validate(source: &str) -> ValidationResult {
    let (compiler, errors, warnings) = run_compiler(source);

    let (ok, rule_count) = if errors.is_empty() {
        let rules = compiler.build();
        (true, rules.iter().count())
    } else {
        (false, 0)
    };

    ValidationResult {
        ok,
        rule_count,
        errors,
        warnings,
    }
}

pub fn compile(source: &str) -> Result<Rules, Vec<Diagnostic>> {
    let (compiler, errors, _) = run_compiler(source);
    if errors.is_empty() {
        Ok(compiler.build())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
rule Demo {
    strings:
        $a = "marker"
    condition:
        $a
}
"#;

    #[test]
    fn valid_source_reports_ok_and_rule_count() {
        let result = validate(VALID);
        assert!(result.ok);
        assert_eq!(result.rule_count, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn syntax_error_lands_on_the_right_line() {
        let source = "rule Broken {\n    condition:\n        $missing\n}";
        let result = validate(source);
        assert!(!result.ok);
        let err = &result.errors[0];
        assert_eq!(err.line, 3);
        assert!(err.from < err.to);
    }

    #[test]
    fn spans_are_utf16_offsets_after_non_ascii_text() {
        // The comment contains Cyrillic (2 bytes per char in UTF-8, 1 code
        // unit in UTF-16), which pushes byte and UTF-16 offsets apart.
        let source = "// правило\nrule Broken {\n    condition:\n        $x\n}";
        let result = validate(source);
        assert!(!result.ok);

        let err = &result.errors[0];
        let expected = source
            .chars()
            .take_while(|c| *c != '$')
            .map(char::len_utf16)
            .sum::<usize>();
        assert_eq!(err.from, expected);
    }

    #[test]
    fn warnings_do_not_block_compilation() {
        let source =
            "rule W {\n    strings:\n        $a = \"ab\"\n    condition:\n        $a or true\n}";
        let result = validate(source);
        assert!(result.ok);
        assert!(!result.warnings.is_empty());
        assert_eq!(result.warnings[0].level, DiagnosticLevel::Warning);
    }

    #[test]
    fn compile_returns_usable_rules() {
        assert!(compile(VALID).is_ok());
        assert!(compile("rule {").is_err());
    }
}
