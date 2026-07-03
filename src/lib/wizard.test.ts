import { describe, expect, it } from "vitest";
import { buildRule } from "./wizard";
import type { CandidateString, SampleAnalysis } from "./api";

function candidate(partial: Partial<CandidateString> & { value: string }): CandidateString {
  return {
    kind: "ascii",
    offset: 0,
    count: 1,
    category: "plain",
    score: 10,
    ...partial,
  };
}

function analysis(partial: Partial<SampleAnalysis> = {}): SampleAnalysis {
  return {
    fileName: "evil-dropper.bin",
    size: 1024,
    sha256: "a".repeat(64),
    entropy: 6.5,
    fileType: "PE executable (MZ)",
    headerHex: "4D 5A 90 00",
    strings: [],
    ...partial,
  };
}

describe("buildRule", () => {
  it("produces a compilable-looking rule with PascalCase name", () => {
    const rule = buildRule(
      analysis(),
      [candidate({ value: "http://c2.example.com", category: "url" })],
      "any",
    );
    expect(rule).toContain("rule EvilDropper");
    expect(rule).toContain('$s01 = "http://c2.example.com"');
    expect(rule).toContain("uint16(0) == 0x5A4D and any of them");
    expect(rule).toContain(`hash        = "${"a".repeat(64)}"`);
  });

  it("escapes quotes and backslashes in string values", () => {
    const rule = buildRule(
      analysis({ fileType: null }),
      [candidate({ value: 'C:\\evil\\"x".exe', category: "path" })],
      "all",
    );
    expect(rule).toContain('$s01 = "C:\\\\evil\\\\\\"x\\".exe" ascii');
    expect(rule).toContain("        all of them");
    expect(rule).not.toContain("uint16");
  });

  it("adds wide modifier for utf-16 strings", () => {
    const rule = buildRule(
      analysis({ fileType: null }),
      [candidate({ value: "WideString", kind: "wide", category: "plain" })],
      "any",
    );
    expect(rule).toContain('$s01 = "WideString" wide ascii');
  });

  it("supports an N-of-them condition and pads identifiers", () => {
    const strings = Array.from({ length: 3 }, (_, i) =>
      candidate({ value: `marker_${i}`, category: "plain" }),
    );
    const rule = buildRule(analysis({ fileType: null }), strings, 2);
    expect(rule).toContain("$s01 =");
    expect(rule).toContain("$s03 =");
    expect(rule).toContain("        2 of them");
  });

  it("uses the ELF magic for ELF samples", () => {
    const rule = buildRule(
      analysis({ fileName: "x.elf", fileType: "ELF executable" }),
      [candidate({ value: "/bin/sh", category: "path" })],
      "any",
    );
    expect(rule).toContain("uint32(0) == 0x464C457F and any of them");
  });

  it("falls back to a safe rule name when the file name has no letters", () => {
    const rule = buildRule(
      analysis({ fileName: "123.bin", fileType: null }),
      [candidate({ value: "abcdef", category: "plain" })],
      "any",
    );
    expect(rule).toMatch(/rule Rule\w+/);
  });
});
