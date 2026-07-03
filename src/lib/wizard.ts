import type { CandidateString, SampleAnalysis } from "./api";

function ruleNameFrom(fileName: string): string {
  const base = fileName.replace(/\.[^.]+$/, "").replace(/[^A-Za-z0-9]+/g, "_");
  const cleaned = base.replace(/^_+|_+$/g, "");
  const named = cleaned.length > 0 ? cleaned : "Sample";
  const prefixed = /^[A-Za-z]/.test(named) ? named : `Rule_${named}`;
  return prefixed
    .split("_")
    .filter(Boolean)
    .map((p) => p[0].toUpperCase() + p.slice(1))
    .join("");
}

function escapeYaraString(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function modifiers(s: CandidateString): string {
  const mods: string[] = [];
  if (s.kind === "wide") mods.push("wide");
  if (s.category === "plain" || s.category === "path" || s.category === "registry") {
    mods.push("ascii");
  }
  return mods.length > 0 ? ` ${mods.join(" ")}` : "";
}

export function buildRule(
  analysis: SampleAnalysis,
  selected: CandidateString[],
  condition: "all" | "any" | number,
): string {
  const name = ruleNameFrom(analysis.fileName);
  const date = new Date().toISOString().slice(0, 10);

  const lines: string[] = [];
  lines.push(`rule ${name}`);
  lines.push("{");
  lines.push("    meta:");
  lines.push(`        author      = "you"`);
  lines.push(`        description = "Auto-generated from ${escapeYaraString(analysis.fileName)}"`);
  lines.push(`        date        = "${date}"`);
  lines.push(`        hash        = "${analysis.sha256}"`);
  if (analysis.fileType) {
    lines.push(`        file_type   = "${escapeYaraString(analysis.fileType)}"`);
  }
  lines.push("");
  lines.push("    strings:");

  const width = Math.max(2, String(selected.length).length);
  selected.forEach((s, i) => {
    const id = `$s${String(i + 1).padStart(width, "0")}`;
    lines.push(`        ${id} = "${escapeYaraString(s.value)}"${modifiers(s)}`);
  });

  lines.push("");
  lines.push("    condition:");

  let cond: string;
  if (analysis.fileType?.startsWith("PE")) {
    cond = "uint16(0) == 0x5A4D and ";
  } else if (analysis.fileType?.startsWith("ELF")) {
    cond = "uint32(0) == 0x464C457F and ";
  } else {
    cond = "";
  }

  if (condition === "all") cond += "all of them";
  else if (condition === "any") cond += "any of them";
  else cond += `${condition} of them`;

  lines.push(`        ${cond}`);
  lines.push("}");
  lines.push("");
  return lines.join("\n");
}
