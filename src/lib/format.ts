export function humanSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = "B";
  for (const next of units) {
    if (value < 1024) break;
    value /= 1024;
    unit = next;
  }
  return `${value.toFixed(value < 10 ? 2 : 1)} ${unit}`;
}

export function hexOffset(offset: number): string {
  return `0x${offset.toString(16).toUpperCase().padStart(8, "0")}`;
}

export function shortHash(sha256: string): string {
  return `${sha256.slice(0, 12)}…${sha256.slice(-6)}`;
}

export function formatTime(epochMs: number): string {
  return new Date(epochMs).toLocaleTimeString(undefined, { hour12: false });
}
