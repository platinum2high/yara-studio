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
  durationMs: number;
  scannedFiles: number;
  matchedFiles: number;
  errorFiles: number;
  cleanFiles: number;
  ruleCount: number;
  cancelled: boolean;
  truncated: boolean;
  results: FileResult[];
}

export interface ScanProgress {
  scanId: string;
  scanned: number;
  matched: number;
  currentPath: string;
}

export function validateRules(source: string): Promise<ValidationResult> {
  return invoke("validate_rules", { source });
}

export function scanPaths(
  source: string,
  libraryRels: string[],
  paths: string[],
  scanId: string,
): Promise<ScanReport> {
  return invoke("scan_paths", { source, libraryRels, paths, scanId });
}

export function cancelScan(scanId: string): Promise<void> {
  return invoke("cancel_scan", { scanId });
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

export interface LibraryEntry {
  rel: string;
  fileName: string;
  ruleNames: string[];
  tags: string[];
  description: string | null;
  compiles: boolean;
  modifiedEpochMs: number;
}

export interface LibraryCollection {
  name: string;
  entries: LibraryEntry[];
}

export interface LibraryTree {
  entries: LibraryEntry[];
  collections: LibraryCollection[];
}

export function libraryList(): Promise<LibraryTree> {
  return invoke("library_list");
}

export function librarySave(
  collection: string | null,
  name: string,
  source: string,
): Promise<string> {
  return invoke("library_save", { collection, name, source });
}

export function libraryRead(rel: string): Promise<string> {
  return invoke("library_read", { rel });
}

export function libraryDelete(rel: string): Promise<void> {
  return invoke("library_delete", { rel });
}

export function libraryCreateCollection(name: string): Promise<void> {
  return invoke("library_create_collection", { name });
}

export function libraryDeleteCollection(name: string): Promise<void> {
  return invoke("library_delete_collection", { name });
}
