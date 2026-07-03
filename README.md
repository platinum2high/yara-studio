<div align="center">

<img src="src-tauri/icons/128x128.png" width="88" alt="YARA Studio icon" />

# YARA Studio

**Offline desktop studio for writing, testing and managing YARA rules.**

Write a rule, drop a sample on it, and see exactly which bytes matched —
without uploading anything, anywhere.

[![CI](https://github.com/platinum2high/yara-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/platinum2high/yara-studio/actions/workflows/ci.yml)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-2b3648)
![Engine](https://img.shields.io/badge/engine-YARA--X%201.19-e8b339)
![License](https://img.shields.io/badge/license-MIT-3fb950)

</div>

<p align="center">
  <img src="docs/screenshots/overview.png" width="900" alt="YARA Studio — editor, live validation and scan results" />
</p>

---

## Why

Analysts write YARA rules all day, but the loop is clumsy: edit a rule in one
tool, run `yara` in a terminal, eyeball the output, guess which string fired,
repeat. Online sandboxes close the loop but you have to **upload the sample** —
a non-starter for anything sensitive.

YARA Studio is that loop in one **100% offline** desktop app. No APIs, no
telemetry, no cloud. Your rules and your samples never leave the machine.

## Features

- **Rule editor** — full YARA syntax highlighting, autocomplete, and live
  validation. Compile errors are underlined as you type, with the exact line
  and message straight from the compiler.
- **Drag & drop scanning** — drop files *or whole directories* onto the app.
  Trees are walked recursively with a live progress counter and a cancel button.
- **Precise match display** — every matched string with its offset, matched
  bytes (hex + ASCII) in context, and the XOR key when the `xor` modifier fired.
- **In-context hex view** — click any match to open a hex dump of the
  surrounding file region with the matched bytes highlighted.
- **Rule library** — save rules into taggable collections, filter by name /
  rule / tag / description. Stored as plain `.yar` files you can keep in git.
- **Multi-rule scanning** — tick library rules to compile them into the scan
  set alongside the editor; each match shows which file the rule came from.
- **Regression tests** — attach *must-match* and *must-not-match* samples to a
  rule; one click re-runs the whole library and flags any rule that drifted.
- **Rule wizard** — point it at a sample; it extracts and ranks strings by IOC
  value and builds a starting rule from your selection.
- **Reports** — export any scan as JSON, CSV (one row per string match,
  SIEM-import friendly) or plain text. SHA-256 for every file, click to copy.
- **Dark theme**, resizable panes, keyboard-first (`⌘/Ctrl+S` to save).

### Inspect matched bytes in file context

<p align="center">
  <img src="docs/screenshots/hex-view.png" width="900" alt="Hex view of a match with the matched bytes highlighted" />
</p>

### Bootstrap a rule from a sample

<p align="center">
  <img src="docs/screenshots/wizard.png" width="900" alt="Rule wizard extracting and categorizing strings from a sample" />
</p>

## Install

Download the installer for your OS from the
[Releases](https://github.com/platinum2high/yara-studio/releases) page:

| OS | Artifact |
|----|----------|
| **macOS** | `.dmg` — universal (Apple Silicon + Intel) |
| **Windows** | `.msi` |
| **Linux** | `.AppImage` and `.deb` |

It's a single self-contained executable — download and double-click, like a
game. No Python, Node, or Docker to install.

> The binaries are not code-signed yet. On macOS use right-click → **Open** on
> first launch; on Windows click **More info → Run anyway** in SmartScreen.

## Engine

YARA Studio embeds [YARA-X](https://virustotal.github.io/yara-x/) — the
official Rust implementation of YARA by VirusTotal — statically compiled into
the binary. All standard modules ship with it: `pe`, `elf`, `macho`, `dotnet`,
`math`, `hash`, `string`, `time`, `lnk`, `dex`, `zip` and more.

Rule compatibility is the project's biggest risk, so it is pinned by tests:
[`src-tauri/tests/compat.rs`](src-tauri/tests/compat.rs) exercises the language
surface the app relies on — string modifiers, hex patterns with jumps and
alternatives, regexes, match counters/offsets, and module functions.

## Development

Requirements: [Rust](https://rustup.rs/) and [Node.js](https://nodejs.org/) 22+.

```sh
npm install
npm run tauri dev     # run the app in dev mode
npm run tauri build   # build a release bundle for the current OS
```

Tests:

```sh
cd src-tauri && cargo test   # engine: compiler, scanner, library, export, tests, wizard, compat
npm test                     # frontend: YARA tokenizer + rule generator
```

Cross-platform CI (fmt + clippy + tests + build) runs on **Ubuntu, macOS and
Windows** for every push. Tagging `v*` builds installers for all three OSes.

## Stack

| Layer | Technology |
|-------|------------|
| Scan engine | [yara-x](https://crates.io/crates/yara-x) (Rust, statically linked) |
| Desktop shell | [Tauri 2](https://tauri.app/) |
| UI | Svelte 5 + TypeScript |
| Editor | CodeMirror 6 with a hand-written YARA language mode |

## License

[MIT](LICENSE)
