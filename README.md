# YARA Studio

Offline desktop studio for writing, testing and managing YARA rules.
Built for SOC analysts and malware researchers who need a fast feedback loop:
write a rule, drop a sample on it, see exactly what matched — without
uploading anything anywhere.

**100% offline.** No APIs, no telemetry, no cloud. Your rules and your
samples never leave the machine.

## Features

- **Rule editor** — full YARA syntax highlighting, autocomplete and live
  validation: compile errors are underlined as you type, with the exact line
  and message from the compiler
- **Drag & drop scanning** — drop files or whole directories onto the app;
  directory trees are walked recursively with live progress and a cancel
  button
- **Precise match display** — every matched string with its offset, matched
  bytes (hex + ASCII) in context, and the XOR key when the `xor` modifier
  fired; click any match to open a hex view of the surrounding file region
- **Rule library** — save rules into collections, filter by name / rule /
  tag / description; stored as plain `.yar` files you can keep in git
- **Multi-rule scanning** — tick library entries to compile them into the
  scan set together with the editor rule; matches show which file the rule
  came from
- **Reports** — export any scan as JSON, CSV (one row per string match,
  SIEM-import friendly) or plain text
- **SHA-256 everywhere** — every scanned file is hashed, click to copy

## Install

Grab the installer for your OS from
[Releases](https://github.com/platinum2high/yara-studio/releases):
`.dmg` (macOS, universal), `.msi` (Windows), `.AppImage` / `.deb` (Linux).

The binaries are not code-signed yet. On macOS use right-click → Open on
first launch; on Windows click "More info" → "Run anyway" in SmartScreen.

## Engine

YARA Studio uses [YARA-X](https://virustotal.github.io/yara-x/), the official
Rust implementation of YARA by VirusTotal, statically compiled into the
application binary. All standard modules are available: `pe`, `elf`, `macho`,
`dotnet`, `math`, `hash`, `string`, `time`, `lnk`, `dex`, `zip` and more.

YARA-X is designed to be compatible with classic YARA rules; the test suite
in `src-tauri/tests/compat.rs` exercises the language surface this project
relies on (string modifiers, hex patterns with jumps and alternatives,
regexes, match counters and offsets, module functions).

## Development

Requirements: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) 22+.

```sh
npm install
npm run tauri dev     # run the app in dev mode
npm run tauri build   # build a release bundle for the current OS
```

Tests:

```sh
cd src-tauri && cargo test   # engine: compiler, scanner, library, export, compat
npm test                     # frontend: YARA tokenizer
```

## Stack

| Layer | Technology |
|---|---|
| Scan engine | [yara-x](https://crates.io/crates/yara-x) (Rust, statically linked) |
| Desktop shell | [Tauri 2](https://tauri.app/) |
| UI | Svelte 5 + TypeScript |
| Editor | CodeMirror 6 with a custom YARA language mode |

## License

[MIT](LICENSE)
