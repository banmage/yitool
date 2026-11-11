# Tasks: 阶段 2 — 焦氏易林索引骨架与查询联动

**Input**: [spec.md](./spec.md), [plan.md](./plan.md)

## Phase 1: Documentation and decisions

- [ ] T001 Record `docs/adr/ADR-025-jiaolin-index-structure.md` for the structure-first `JL-01` strategy, stable `UPPER/LOWER` hex ID, `PENDING` policy, and `DB-v0.4`.
- [ ] T002 Update `docs/02-DECISION-REGISTER.md` with the structural decision and keep D-28/D-29 explicitly open.
- [ ] T003 Update `docs/04-DATA-DICTIONARY.md` with `JL-01-v0.1`, `PENDING`, resource path, and `DB-v0.4` fields.

## Phase 2: Foundational (blocking prerequisites)

- [ ] T004 Add `crates/jiaolin` to `Cargo.toml` and create `crates/jiaolin/Cargo.toml`, `src/lib.rs`, `src/table.rs`, `src/error.rs`.
- [ ] T005 [P] Add `Trigram::from_key`, `Hexagram::from_key`, and `Hexagram::from_name` in `crates/domain/src/hexagram.rs` with 64-key/name round-trip tests.
- [ ] T006 [P] Add `scripts/gen_jiaolin_skeleton.py`; generate `resources/data/jiaolin_4096.json` from `resources/data/hex64.json` with all 4096 entries `PENDING` and no verse text.
- [ ] T007 Add `data/jiaolin_4096.json: JL-01` to `scripts/gen_manifest.py`, regenerate `resources/manifests/data.manifest.json`, and verify checksum.
- [ ] T008 [P] Add `tests/golden/jl01_queries.json` with at least the `天地否 → 天雷无妄` pair, one `origin==target` pair, and one reverse-pair non-match.

## Phase 3: User Story 1 — 4096 键索引骨架 (P1)

**Goal**: Load and validate a complete 4096-key skeleton without inventing text.

**Independent Test**: `cargo test -p yitool-jiaolin`; all integrity/tamper/cross-check tests pass.

- [ ] T009 [P] [US1] Add `crates/jiaolin/tests/integrity.rs`: 4096 unique keys, 64×64 coverage, 64 diagonal entries, valid statuses.
- [ ] T010 [P] [US1] Add `crates/jiaolin/tests/tamper.rs`: duplicate key, missing pair, unknown hex ID, invalid status, empty `VERIFIED` text/edition/source are rejected.
- [ ] T011 [P] [US1] Add `crates/jiaolin/tests/hex_crosscheck.rs`: every ID/name matches `HEX-01` upper/lower trigrams.
- [ ] T012 [US1] Implement `JiaolinEntry`, `CollationStatus`, `JiaolinTable::load`, and the `HashMap` key index in `crates/jiaolin/src/table.rs`.
- [ ] T013 [US1] Implement resource metadata/version checks and `VERIFIED` required-field validation in `crates/jiaolin/src/table.rs`.
- [ ] T014 [US1] Add a resource test proving the committed skeleton is all `PENDING` and contains no fabricated verse text.

## Phase 4: User Story 2 — 从梅花结果查询焦氏条目 (P1)

**Goal**: Resolve `(origin, target)` from a Meihua result or manual selection with no fallback.

**Independent Test**: `cargo run -p yitool-cli -- jiaolin --origin "天地否" --target "天雷无妄"` returns `(QIAN/KUN, QIAN/ZHEN)` with `PENDING`.

- [ ] T015 [P] [US2] Add `crates/jiaolin/tests/query.rs`: diagonal lookup, ordered pair, reverse-pair non-match, missing key.
- [ ] T016 [US2] Implement `JiaolinTable::lookup`, `JiaolinLookup`, `LookupStatus`, and `index_hint` in `crates/jiaolin/src/table.rs`/`lib.rs`.
- [ ] T017 [US2] Add `jiaolin` to `crates/app/src/context.rs`: load `JL-01` at startup and expose `AppContext::jiaolin_query`.
- [ ] T018 [US2] Add `Command::Jiaolin` and dispatch in `crates/cli/src/main.rs`, accepting names, `UPPER/LOWER` keys, optional target, and `--href`.
- [ ] T019 [US2] Add CLI JSON/text output and argument errors for unknown names/keys and invalid hrefs.

## Phase 5: User Story 3 — 原文、白话与校勘状态分离展示 (P1)

**Goal**: Show `PENDING`/`MISSING` safely, quote `VERIFIED` text verbatim, and label system-written notes.

**Independent Test**: Report and CLI show “待校勘” for `PENDING`/missing, and a `VERIFIED` fixture shows exact text plus “本系统白话提示，非原文”.

- [ ] T020 [P] [US3] Add `crates/ir/src/jiaolin.rs` with `PLAIN_NOTE_LABEL`, `pending_message`, `index_hint`, and status labels; add unit tests.
- [ ] T021 [US3] Implement the IR wording module and export it from `crates/ir/src/lib.rs`.
- [ ] T022 [US3] Add the “焦氏易林参考” section and `JL-01` version row to `crates/app/src/report.rs`.
- [ ] T023 [US3] Add `crates/app/tests/jiaolin.rs` for `PENDING`, `MISSING`, and `VERIFIED` report rendering, including the plain-note label.
- [ ] T024 [P] [US3] Add link-resolver tests for `yitool://jiaolin/QIAN-KUN/QIAN-ZHEN`, invalid slugs, missing entries, and PENDING entries.
- [ ] T025 [US3] Implement `crates/app/src/jiaolin.rs` href parsing/resolution and export it from `crates/app/src/lib.rs`.
- [ ] T026 [US3] Add a compliance test proving the scanner targets system-generated text and does not rewrite the quoted original.

## Phase 6: User Story 4 — 增量校勘与版本化 (P2)

**Goal**: Persist the JL version and lookup with history; support later `VERIFIED` upgrades without silent recomputation.

**Independent Test**: Save a run with `JL-01-v0.1`; upgrade to `JL-01-v0.2`; old history still renders the saved version and `PENDING`.

- [ ] T027 [P] [US4] Add `crates/store` migration tests for `DB-v0.3 → DB-v0.4`, preserving old `run_reports` rows with `NULL` `jl_*` columns.
- [ ] T028 [US4] Implement `DB-v0.4` migration (`jl_version`, `jl_json`) and update `SCHEMA_VERSION` in `crates/store/src/lib.rs`.
- [ ] T029 [US4] Add `jl_version`/`jl_json` to `RunReport`; update `save_run_report`/`get_run_report` and all constructors.
- [ ] T030 [US4] Add `jiaolin` to `CastingPayload` and save the lookup in `perform_cast_with_store` without blocking Meihua on JL failure.
- [ ] T031 [US4] Add `crates/app/tests/history.rs` coverage for JL history reload and old-version rendering.
- [ ] T032 [US4] Implement `VERIFIED` upgrade validation (text, edition, source_ref, explicit variants) and document the manual-sampling requirement.

## Phase 7: User Story 5 — 阶段 2 验收入口 (P1)

**Goal**: Desktop read-only panel, manual query, offline/privacy acceptance, and full quality gate.

**Independent Test**: `./scripts/check.sh`; frontend tests cover auto-linkage, manual query, `PENDING`, `VERIFIED` fixture, missing link, and history restore.

- [ ] T033 [US5] Add `get_jiaolin_options`, `get_jiaolin`, and `resolve_jiaolin` to `apps/desktop/src-tauri/src/lib.rs`; register them in `main.rs`.
- [ ] T034 [US5] Add `JiaolinEntry`/`JiaolinLookup`/`JiaolinOptions` and `VersionSet.jl_data` to `apps/desktop/src/types.ts`; add API wrappers to `apps/desktop/src/api.ts`.
- [ ] T035 [US5] Create `apps/desktop/src/JiaolinPanel.tsx` with status, index hint, text/edition/variants, plain-note label, manual selectors, and “待校勘” states.
- [ ] T036 [US5] Integrate the panel into `apps/desktop/src/App.tsx` and restore JL from `jl_json` for history details without re-querying.
- [ ] T037 [US5] Add frontend tests for auto-linkage, manual query, `PENDING`, `VERIFIED` fixture, missing link, and history restore.
- [ ] T038 [US5] Add offline/privacy acceptance: no network calls, no birth/question plaintext, and `privacy-scan` 0 hits.
- [ ] T039 [US5] Run CLI acceptance and `./scripts/check.sh`; record results in the feature notes.

## Phase 8: Polish and status

- [ ] T040 Update `README.md`, `docs/00-PROJECT-ROADMAP.md`, and `AGENTS.md` with the `jiaolin` command, `JL-01-v0.1` status, and D-28/O-06 blocker.
- [ ] T041 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and the full quality gate.
- [ ] T042 Record phase 2 structural completion while keeping `JL-01` and phase 0 open; do not claim 4096 texts are collated.

## Blocked / Follow-up

- [ ] T043 [BLOCKED: D-28/O-06] Collate the 4096 `JL-01` entries against a frozen base edition; bump `JL-01` version and checksum incrementally.
- [ ] T044 [BLOCKED: D-29] Review and approve final plain-note wording; `PENDING` entries must remain non-deterministic until then.
- [ ] T045 Verify the Windows NSIS package from phase 1 on a Windows machine (carry-over, not part of phase 2).
