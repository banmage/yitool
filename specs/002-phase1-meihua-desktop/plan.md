# Implementation Plan: 阶段 1 — 梅花单引擎桌面闭环

**Branch**: `002-phase1-meihua-desktop` | **Date**: 2026-11-06 | **Spec**: [spec.md](./spec.md)

## Summary

Build a local Tauri 2 desktop shell with a React/TypeScript interface. The UI submits the nine-color form to Rust commands, which call the existing `AppContext` for calendar lookup, Meihua calculation, report rendering, and allowlisted evidence resolution. The result view, report view, copy, and download share the Rust-generated Markdown string. No user inputs or results are persisted.

## Technical Context

**Language/Version**: Rust stable; TypeScript/React  
**Primary Dependencies**: Tauri 2, Vite, React, `@tauri-apps/api`; existing Rust `app`, `cal`, `meihua` crates  
**Storage**: None; application resources remain read-only  
**Testing**: Rust command/contract tests, existing workspace suite, frontend typecheck/build, manual desktop smoke checklist  
**Target Platform**: Tauri desktop; start with Linux development, preserve Tauri cross-platform configuration  
**Constraints**: Offline; no persistence; no internet URL resolution; retain pending status; stage 1 only.

## Constitution Check

- **Privacy**: Form/result state remains transient in the frontend and Rust command stack; no logs or database writes.
- **Traceability**: All computation, report generation, and evidence lookup use existing Rust services and verified local resources.
- **Compliance**: Always show the core disclaimer and PENDING status; no total score or absolute conclusion.
- **Phase scope**: This feature implements only the stage 1 nine-color Meihua workflow. It does not claim that JL-01 is verified or phase 0 is fully closed.
- **Quality gate**: Preserve Rust workspace tests and add direct desktop command/UI contract validation.

## Project Structure

```text
apps/desktop/
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── api.ts
│   ├── types.ts
│   └── styles.css
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    └── src/main.rs
specs/002-phase1-meihua-desktop/
├── spec.md
├── plan.md
├── tasks.md
└── checklists/requirements.md
```

## Implementation Approach

1. Add the separate Tauri crate and package scripts without adding the GUI to the phase 0 Rust workspace.
2. Initialize the verified resource-backed `AppContext` once at desktop startup; fail startup explicitly if resources cannot be loaded.
3. Implement a typed Tauri command that validates the form, performs the CAL lookup and Meihua cast, and returns result, versions, and the one generated Markdown report.
4. Implement a second command that resolves only the existing `yitool://` evidence index and returns the read-only detail.
5. Build the React form, result/steps display, report dialog with same-string copy/download, and accessible evidence reader.
6. Document local setup and commands; add the smallest applicable checks and run Rust workspace plus frontend checks where the installed toolchain permits.

## Risks

- GUI toolchain packages (Node, WebKitGTK, appindicator) may not be installed; phase 0 checks must remain independent of them.
- Resource lookup must work both in development and from a bundled Tauri application. Packaging must include the same verified `resources/` tree.
- Tauri configuration and Linux WebKit dependencies vary by host; startup/build failures must remain explicit rather than falling back to unverified data.
