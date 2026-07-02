import { describe, expect, it } from "vitest";
import { StringStream } from "@codemirror/language";
import { yaraStreamParser } from "./language";

interface Token {
  text: string;
  style: string | null;
}

function tokenize(source: string): Token[] {
  const state = yaraStreamParser.startState!(4);
  const tokens: Token[] = [];

  for (const line of source.split("\n")) {
    if (line === "") {
      yaraStreamParser.blankLine?.(state, 4);
      continue;
    }
    const stream = new StringStream(line, 4, 4);
    while (!stream.eol()) {
      const start = stream.pos;
      stream.start = start;
      const style = yaraStreamParser.token(stream, state);
      if (stream.pos === start) throw new Error(`tokenizer stuck at "${line.slice(start)}"`);
      tokens.push({ text: line.slice(start, stream.pos), style: style ?? null });
    }
  }
  return tokens;
}

function styleOf(tokens: Token[], text: string): string | null | undefined {
  return tokens.find((t) => t.text === text)?.style;
}

describe("yara tokenizer", () => {
  it("highlights declaration keywords and rule names", () => {
    const tokens = tokenize("private rule EvilDropper : apt {");
    expect(styleOf(tokens, "private")).toBe("keyword");
    expect(styleOf(tokens, "rule")).toBe("keyword");
    expect(styleOf(tokens, "EvilDropper")).toBe("typeName.definition");
    expect(styleOf(tokens, "apt")).toBe("atom");
  });

  it("highlights string identifiers with all sigils", () => {
    const tokens = tokenize("condition:\n    $a and #b > 2 and @c[1] < 100 and !d == 5");
    for (const id of ["$a", "#b", "@c", "!d"]) {
      expect(styleOf(tokens, id)).toBe("variableName.special");
    }
  });

  it("tokenizes text strings with escapes as one string token", () => {
    const tokens = tokenize('$s = "evil \\"quoted\\" \\x90 payload"');
    const str = tokens.find((t) => t.style === "string");
    expect(str?.text).toBe('"evil \\"quoted\\" \\x90 payload"');
  });

  it("enters and leaves hex string mode", () => {
    const tokens = tokenize("$h = { 4D 5A ?? [4-8] ( AA | BB ) 90 }\n$t = \"after\"");
    expect(styleOf(tokens, "4D")).toBe("number");
    expect(styleOf(tokens, "??")).toBe("atom");
    expect(styleOf(tokens, "[4-8]")).toBe("meta");
    expect(styleOf(tokens, "|")).toBe("operator");
    expect(styleOf(tokens, '"after"')).toBe("string");
  });

  it("does not treat rule body braces as hex strings", () => {
    const tokens = tokenize("rule X {\n    condition:\n        true\n}");
    expect(styleOf(tokens, "true")).toBe("bool");
  });

  it("tokenizes regexes with flags and modifiers after them", () => {
    const tokens = tokenize("$re = /c2\\.[a-z]{3}\\/path/is nocase");
    const re = tokens.find((t) => t.style === "regexp");
    expect(re?.text).toBe("/c2\\.[a-z]{3}\\/path/is");
    expect(styleOf(tokens, "nocase")).toBe("modifier");
  });

  it("carries block comments across lines", () => {
    const tokens = tokenize("/* line one\nstill comment */ rule");
    expect(tokens.filter((t) => t.style === "comment").length).toBeGreaterThanOrEqual(2);
    expect(styleOf(tokens, "rule")).toBe("keyword");
  });

  it("highlights module names only in import or dotted use", () => {
    const imported = tokenize('import "pe"');
    expect(styleOf(imported, "pe")).toBeUndefined();

    const dotted = tokenize("condition:\n    pe.is_pe and math.entropy(0, filesize) > 7");
    expect(styleOf(dotted, "pe")).toBe("className");
    expect(styleOf(dotted, "math")).toBe("className");
    expect(styleOf(dotted, "filesize")).toBe("bool");
  });

  it("recognizes numbers in all YARA forms", () => {
    const tokens = tokenize("condition:\n    filesize < 5MB and uint16(0) == 0x5A4D and #a == 12");
    expect(styleOf(tokens, "5MB")).toBe("number");
    expect(styleOf(tokens, "0x5A4D")).toBe("number");
    expect(styleOf(tokens, "12")).toBe("number");
    expect(styleOf(tokens, "uint16")).toBe("variableName.standard");
  });

  it("highlights operator keywords and modifiers", () => {
    const tokens = tokenize('$a = "x" wide xor(0x01-0xff)\ncondition:\n    all of them at 0');
    expect(styleOf(tokens, "wide")).toBe("modifier");
    expect(styleOf(tokens, "xor")).toBe("modifier");
    expect(styleOf(tokens, "all")).toBe("operatorKeyword");
    expect(styleOf(tokens, "of")).toBe("operatorKeyword");
    expect(styleOf(tokens, "them")).toBe("operatorKeyword");
    expect(styleOf(tokens, "at")).toBe("operatorKeyword");
  });
});
