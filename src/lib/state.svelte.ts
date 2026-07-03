import { SvelteSet } from "svelte/reactivity";
import type { EditorView } from "@codemirror/view";
import type {
  LibraryTestReport,
  LibraryTree,
  ScanProgress,
  ScanReport,
  ValidationResult,
} from "./api";

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

export const NEW_RULE = `rule NewRule
{
    meta:
        author      = ""
        description = ""

    strings:
        $a = ""

    condition:
        $a
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
  scanId = $state<string | null>(null);
  scanProgress = $state<ScanProgress | null>(null);
  scanSet = new SvelteSet<string>();
  dragActive = $state(false);

  libraryOpen = $state(true);
  libraryTree = $state<LibraryTree | null>(null);
  currentRel = $state<string | null>(null);
  savedSource = $state<string | null>(null);
  saveDialogOpen = $state(false);
  wizardOpen = $state(false);
  testsRel = $state<string | null>(null);
  testsReport = $state<LibraryTestReport | null>(null);
  flash = $state<string | null>(null);

  dirty = $derived(this.currentRel !== null && this.source !== this.savedSource);

  // Not reactive on purpose: the CodeMirror view is an imperative handle,
  // not renderable state.
  editorView: EditorView | null = null;

  setEditorContent(text: string) {
    const view = this.editorView;
    if (!view) return;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: text },
    });
    this.source = text;
  }

  showFlash(message: string) {
    this.flash = message;
    setTimeout(() => (this.flash = null), 2000);
  }
}

export const app = new AppState();
