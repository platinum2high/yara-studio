# YARA Studio

Offline desktop studio for writing, testing and managing YARA rules.
Built for SOC analysts and malware researchers who need a fast feedback loop:
write a rule, drop a sample on it, see exactly what matched — without
uploading anything anywhere.

**100% offline.** No APIs, no telemetry, no cloud. Your rules and your
samples never leave the machine.

## Features

- **Rule editor** with full YARA syntax highlighting and live validation —
  compile errors are underlined in the editor as you type, with the exact
  line and message from the compiler
- **Drag & drop scanning** — drop files onto the app, they are scanned
  against the rules in the editor
- **Precise match display** — every matched string with its offset, matched
  bytes (hex + ASCII) and XOR key when the `xor` modifier fired

Planned next: hex view of matches in file context, rule library with
collections and tags, recursive directory scanning, JSON/CSV/TXT reports,
per-rule regression testing against known-good samples.

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

Rust engine tests:

```sh
cd src-tauri && cargo test
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
