# Contributing to YARA Studio

Thanks for your interest in contributing! 🎉 This project is built for SOC
analysts and malware researchers, and real-world input from people who do the
work is exactly what makes it better.

## Ways to help

- 💡 **Ideas & feature requests** — open a
  [Discussion](https://github.com/platinum2high/yara-studio/discussions) or an
  Issue. If you have a workflow in mind (SOC, MSSP, DFIR), describe it — that
  context is gold.
- 🐛 **Bug reports** — open an Issue with steps to reproduce, your OS, and what
  you expected.
- 🔧 **Code** — pick an open Issue (look for
  [`good first issue`](https://github.com/platinum2high/yara-studio/labels/good%20first%20issue)),
  or propose a change in an Issue first so we can align before you build.

## Getting started

Requirements: [Rust](https://rustup.rs/) and [Node.js](https://nodejs.org/) 22+.

```sh
git clone https://github.com/platinum2high/yara-studio.git
cd yara-studio
npm install
npm run tauri dev
```

Project layout:

- `src-tauri/` — Rust backend: the YARA engine wrappers, scanning, library,
  tests, wizard. Business logic and unit tests live here.
- `src/` — Svelte 5 + TypeScript frontend: editor, results, dialogs.

## Before you open a pull request

Please make sure the checks that CI runs pass locally:

```sh
# Rust
cd src-tauri
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend
cd ..
npm run check
npm test
```

Guidelines:

- Keep changes focused — one feature or fix per PR.
- Match the surrounding code style. Let good names document the code; only
  comment *why* when it would surprise a future reader.
- Add tests for new logic (the engine side is well covered — keep it that way).
- No half-built features: if it's in the UI, it should work.

## Contributor License Agreement (CLA)

YARA Studio is **dual-licensed** — AGPL-3.0 for open use, plus a commercial
license for closed-source embedding (see [COMMERCIAL.md](COMMERCIAL.md)).

For that model to keep working, every contribution needs to be covered by the
[Contributor License Agreement](CLA.md). It's short: you keep the copyright to
your work, and you grant the maintainer permission to distribute it under both
the AGPL **and** the commercial license.

**By opening a pull request, you agree to the terms in [CLA.md](CLA.md).**
Please add this line to your PR description to confirm:

```
I have read and agree to the CLA.
```

## Questions

Not sure where to start, or want to talk through an idea before coding? Open a
[Discussion](https://github.com/platinum2high/yara-studio/discussions) — happy
to help.
