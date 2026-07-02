import { invoke } from "@tauri-apps/api/core";

export interface Diagnostic {
  level: "error" | "warning";
  code: string;
  title: string;
  detail: string;
  line: number;
  column: number;
  from: number;
  to: number;
}

export interface ValidationResult {
  ok: boolean;
  ruleCount: number;
  errors: Diagnostic[];
  warnings: Diagnostic[];
}

export interface StringMatch {
  identifier: string;
  offset: number;
  length: number;
  matchedHex: string;
  matchedAscii: string;
  contextBeforeHex: string;
  contextAfterHex: string;
  xorKey: number | null;
  truncated: boolean;
}

export interface RuleMatch {
  identifier: string;
  namespace: string;
  tags: string[];
  meta: Record<string, unknown>;
  stringMatches: StringMatch[];
}

export interface FileResult {
  path: string;
  fileName: string;
  size: number;
  sha256: string | null;
  durationMs: number;
  status: "matched" | "clean" | "error";
  error: string | null;
  ruleMatches: RuleMatch[];
}

export interface ScanReport {
  startedAtEpochMs: number;
  totalFiles: number;
  matchedFiles: number;
  errorFiles: number;
  ruleCount: number;
  results: FileResult[];
}

export function validateRules(source: string): Promise<ValidationResult> {
  return invoke("validate_rules", { source });
}

export function scanPaths(source: string, paths: string[]): Promise<ScanReport> {
  return invoke("scan_paths", { source, paths });
}

export interface HexRegion {
  fileSize: number;
  start: number;
  bytesHex: string;
}

export function readHexRegion(
  path: string,
  start: number,
  length: number,
): Promise<HexRegion> {
  return invoke("read_hex_region", { path, start, length });
}
