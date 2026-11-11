# Tasks: 阶段 1 补全

**Input**: [spec.md](./spec.md), [plan.md](./plan.md)

## Phase 1: Documentation and decisions

- [x] T001 Create phase 1 completion specification, plan, and checklist.
- [x] T002 Record ADR-022 for history/profile/settings data strategy (D-23/D-24/D-25).
- [x] T003 Record ADR-023 for birth-data reproducibility wording and nine-color imagery copy (D-22/D-27).
- [x] T004 Update decision register and roadmap for the new phase 1 scope.

## Phase 2: Store and migrations

- [x] T005 Add `DB-v0.3` migration: `run_reports` table and expanded `run_steps` statuses.
- [x] T006 Add store queries for profiles, run records, steps, reports, notes, and settings.
- [x] T007 Add migration/backup/isolation tests for `DB-v0.3`.

## Phase 3: App orchestration

- [x] T008 Add history save/load orchestration and seed summary in reports.
- [x] T009 Add tests for saved report stability, seed chain, and privacy-safe history.

## Phase 4: Desktop backend

- [x] T010 Extend cast request for mode, scheme, traditional numbers, draw, and profile.
- [x] T011 Wire seed chain, store persistence, history/profile/settings commands.
- [x] T012 Add desktop command tests for traditional mode, draw, history isolation, and invalid input.

## Phase 5: Desktop UI

- [x] T013 Add profile selector and settings dialog.
- [x] T014 Add nine-color/traditional mode switch, manual/random draw, and seed summary display.
- [x] T015 Add history list, detail report, and delete flow.
- [x] T016 Update frontend tests for new API surface and workflows.
- [x] T017 Add styles and keyboard/focus behavior for new panels.

## Phase 6: Validation

- [x] T018 Run Rust workspace, desktop command, frontend, and quality-gate checks.
- [x] T019 Record unavailable GUI prerequisites explicitly and update status docs.
