# Tasks: 阶段 1 — 梅花单引擎桌面闭环

**Input**: [spec.md](./spec.md), [plan.md](./plan.md)

## Phase 1: Setup

- [x] T001 Create a separate Tauri 2 crate and React/TypeScript/Vite package under `apps/desktop`.
- [x] T002 Configure development and bundle resource paths so the verified `resources/` directory is read-only and available in both modes.
- [x] T003 Add package scripts for frontend typecheck/build and Tauri development/build.

## Phase 2: Rust desktop commands

- [x] T004 Implement typed nine-color input validation, local-time parsing, and explicit timezone/day-bound handling.
- [x] T005 Initialize `AppContext` once and expose a cast command that returns the CAL result, Meihua result, versions, and one Rust-rendered Markdown report.
- [x] T006 Expose a read-only evidence resolution command using the existing evidence index; reject invalid and unknown hrefs.
- [x] T007 Add backend tests for invalid input, pending state propagation, evidence failures, and output/report consistency.

## Phase 3: Nine-color workflow

- [x] T008 Implement accessible color and zodiac selectors, date/time, timezone, and day-bound inputs.
- [x] T009 Display complete result, calendar context, statuses, versions, and disclaimer without recomputing any business rules in TypeScript.
- [x] T010 Display every step with rule/source/resource links that open the evidence reader.
- [x] T011 Add loading, validation, integrity failure, and command error states.

## Phase 4: Report and evidence reading

- [x] T012 Add a keyboard-accessible report dialog displaying the exact backend Markdown string.
- [x] T013 Implement copy and `.md` download from that same string, with explicit failure feedback and selectable text fallback.
- [x] T014 Implement read-only evidence details with explicit PENDING/missing/checksum states and no external navigation.
- [x] T015 Add frontend checks for form validation, pending warnings, same-string export, and evidence protocol allowlisting.

## Phase 5: Polish and validation

- [x] T016 Add responsive, keyboard-visible, screen-reader-accessible presentation and focus restoration for dialogs.
- [x] T017 Document desktop prerequisites, setup, run, and build commands; update project status without claiming phase 0/JL-01 completion.
- [x] T018 Run frontend checks, targeted desktop command tests, `cargo fmt --all -- --check`, and `./scripts/check.sh`; record unavailable GUI prerequisites explicitly.

**T018 results**: Frontend tests/build/audit, desktop service tests, formatting, and `./scripts/check.sh` pass. The full Tauri target cannot compile on this host because it lacks `pkg-config` and GTK/WebKit development packages; the desktop CI job installs these prerequisites and runs the full Tauri test target.
