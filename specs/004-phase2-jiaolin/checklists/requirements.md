# Specification Quality Checklist: 阶段 2 焦氏易林索引骨架与查询联动

**Purpose**: Validate specification completeness before planning
**Created**: 2026-11-11
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] User scenarios describe user-visible value without prescribing implementation
- [x] Focused on the phase 2 structural scope: 4096 keys, PENDING semantics, query linkage, collation workflow
- [x] User stories, requirements, edge cases, and success criteria are complete
- [x] Compliance, offline, privacy, versioning, and accessibility boundaries are explicit
- [x] The spec does not fabricate or pre-fill 4096 verse texts

## Requirement Completeness

- [x] Stable hexagram ID convention is defined as `UPPER/LOWER`, reusing the existing domain key
- [x] 64×64 coverage, 64 diagonal entries, uniqueness, and missing-key behavior are testable
- [x] `origin==target` and ordered `(origin, target)` lookup rules are explicit
- [x] No-fallback, no-invention, and no-silent-hiding behavior are explicit
- [x] `PENDING`, `VERIFIED`, and query-level `MISSING` semantics are distinguishable
- [x] Original text, edition, variants, and system-written plain note are separated
- [x] Collation upgrade requirements (source, edition, sampling) are testable
- [x] Resource manifest, checksum, version bump, and history replay are covered
- [x] CLI, desktop view, stable links, and accessibility have observable acceptance behavior
- [x] Offline, privacy, and compliance-scan boundaries are covered
- [x] Scope excludes scoring, trend synthesis, and phase 3–5 engines

## Feature Readiness

- [x] Requirements map to SRS phase 2, FR-03, 3.2.2; ARS 1.10/4; IR 4
- [x] Each requirement has observable pass/fail behavior
- [x] Dependencies on `HEX-01`, existing Meihua result, history, and resource manifest are identified
- [x] D-28/O-06 and D-29 are recorded as blockers for bulk text population and final plain-note review
- [x] Phase 2 structural exit conditions can be verified without closing JL-01
- [x] The spec explicitly keeps JL-01 and phase 0 open

## Notes

- This feature delivers the JL-01 skeleton and query linkage only. Actual 4096-text collation remains blocked by D-28/O-06.
- The initial resource may be entirely `PENDING`; a `VERIFIED` fixture is still required to prove the display and validation path.
- Stable hex IDs reuse `Hexagram::key()` (`UPPER/LOWER`); any future change must be versioned and must not rewrite history.
- The desktop view is a read-only phase 2 surface; the full ancient-text reader remains a later-phase extension.
