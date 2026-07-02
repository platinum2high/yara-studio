import type { ScanReport, ValidationResult } from "./api";

export const DEFAULT_RULE = `import "math"

rule SuspiciousBeacon : demo network
{
    meta:
        author      = "you"
        description = "Demo rule — drop a file on the right panel to scan it"
        date        = "2026-07-02"

    strings:
        $ua     = "Mozilla/5.0 (compatible; scanner)" ascii wide
        $host   = /c2\\.[a-z0-9\\-]{4,32}\\.(net|org|top)/ nocase
        $magic  = { 4D 5A 90 00 [4-32] 50 45 00 00 }
        $xored  = "beacon_interval" xor(0x01-0xff)

    condition:
        2 of them and
        filesize < 5MB and
        math.entropy(0, filesize) > 4.0
}
`;

export interface CursorPosition {
  line: number;
  column: number;
}

class AppState {
  source = $state(DEFAULT_RULE);
  validation = $state<ValidationResult | null>(null);
  cursor = $state<CursorPosition>({ line: 1, column: 1 });
  report = $state<ScanReport | null>(null);
  scanning = $state(false);
  scanError = $state<string | null>(null);
  dragActive = $state(false);
}

export const app = new AppState();
