---
date: 2026-07-05 02:09:52 EDT
repo: git@github.com:jmagar/rustifi.git
branch: codex/unifi-api-support
head: a5e5cd4
plan: docs/superpowers/plans/2026-07-04-unifi-official-and-internal-api-support.md
working directory: /home/jmagar/workspace/rustifi/.worktrees/codex-unifi-api-support
worktree: /home/jmagar/workspace/rustifi/.worktrees/codex-unifi-api-support
pr: https://github.com/jmagar/unifi-rmcp/pull/2
beads: rustifi-4wo
---

# UniFi API support session

## User Request

Implement the planned official UniFi API support while preserving verified internal UniFi Network coverage, following the rustifi repo guardrails.

## Session Overview

The branch added registry-backed official API dispatch, verified internal capability exposure, hybrid aliases, tests, and documentation. Review follow-up removed the obsolete boolean mutation gate entirely; mutations are now represented by capability metadata and MCP admin scope.

## Sequence of Events

1. Created and worked in the isolated `codex/unifi-api-support` worktree.
2. Implemented official/internal/hybrid API support through generated inventories and explicit runtime registries.
3. Ran local review agents and fixed findings around unverified internal routes, auth scopes, hybrid defaults, path encoding, query validation, stale docs, and parser coverage.
4. Reworked the mutation contract so mutating actions require `unifi:admin` under mounted MCP auth and no extra action parameter.
5. Ran formatting, clippy, tests, no-gate string sweep, LOC, and `mod.rs` checks.

## Key Findings

- Official API support is registry-backed from `data/unifi_official_network_v10_3_58.json`.
- Internal inventory remains a 180-row reference catalog, but only verified rows are exposed at runtime.
- Hybrid aliases default to internal routes unless `siteId` or `prefer="official"` is supplied.
- Connector proxy paths remain restricted to official integration prefixes.
- Follow-up bead `rustifi-4wo` tracks live verification for the full internal endpoint catalog.

## Technical Decisions

- Keep official endpoint coverage generic instead of hand-maintaining 78 match arms.
- Treat unverified internal rows as research data, not callable runtime capabilities.
- Use `unifi:read` for read actions and `unifi:admin` for mutating actions in mounted MCP auth.
- Remove the boolean mutation gate completely; future interactive safety should use MCP elicitation.
- Keep generated-action parsing strict so malformed params and unknown flags fail early.

## Files Changed

| status | path | purpose | evidence |
|---|---|---|---|
| modified | `src/actions.rs` | Action request and registry dispatch | Mutation gate removed; source family dispatch retained |
| modified | `src/api/http.rs` | Shared HTTP execution | Rejects non-object query and treats empty GET bodies as errors |
| modified | `src/actions/official.rs` | Official path substitution | Scalar path params and percent encoding |
| modified | `src/actions/hybrid.rs` | Hybrid routing | Internal default with official opt-in via `siteId` or preference |
| modified | `src/capabilities/*` | Capability registry | Verified internal filtering and mutating metadata |
| modified | `src/mcp/*` | MCP schema/auth/tool dispatch | Admin scope for mutating actions, no boolean mutation parameter |
| modified | `src/cli.rs` | CLI parser | Real parser is library-visible and rejects malformed generated-action args |
| modified | `tests/*` | Regression coverage | HTTP capture, registry drift, hybrid aliases, parser negatives, auth scope mapping |
| modified | `docs/*`, `README.md`, plugin skill | User-facing docs | Port, binary name, response shape, and mutation contract corrected |
| modified | `data/unifi_internal_reference_tools.json` | Internal reference inventory | Adds verified flags |
| modified | `xtask/src/internal_reference.rs` | Inventory generation | Marks curated verified rows vs unverified reference rows |

## Beads Activity

| id | title | action | status | why |
|---|---|---|---|---|
| `rustifi-4wo` | Verify full internal UniFi action endpoint mappings | Created by implementation worker and reviewed during closeout | open | Tracks verification needed before exposing the full internal reference catalog |

## Repository Maintenance

- Plans: replaced the stale long implementation plan with a concise current plan/status artifact because old snippets described a removed mutation gate.
- Beads: verified `rustifi-4wo` remains open for the only known follow-up.
- Worktrees and branches: work remained on active PR branch `codex/unifi-api-support`; no cleanup was safe while PR #2 is open.
- Stale docs: updated README, plugin skill, runtime usage, coverage docs, and smoke-test diagnostics.

## Tools and Skills Used

- Shell commands: git, cargo, bd, gh, jq/perl-style checks, and filesystem inspection.
- File tools: `apply_patch` for manual source/doc edits.
- Skills: `vibin:work-it` for isolated worktree execution; `vibin:save-to-md` for this session artifact.
- Subagents: implementation worker plus review/simplification agents for code, tests, docs, silent-failure, and type-design checks.
- GitHub CLI: PR status and comment inspection.

## Commands Executed

| command | result |
|---|---|
| `cargo fmt --check` | passed after formatting |
| `cargo clippy --all-targets -- -D warnings` | passed |
| `cargo test` | passed; live smoke tests ignored |
| removed-gate literal sweep | no matches |
| `bd show rustifi-4wo --json` | open follow-up verified |
| `gh pr view 2 --repo jmagar/unifi-rmcp ...` | PR open and mergeable; external review bots rate-limited |

## Errors Encountered

- Review bots on GitHub were quota/rate-limited; local review agents were used instead.
- Initial generated internal inventory exposed unverified synthetic paths; fixed by marking rows verified/unverified and filtering runtime capabilities.
- Initial MCP auth denied generated read actions; fixed by deriving scopes from capability metadata.
- The mutation gate was initially retained; removed completely after user correction.

## Behavior Changes

| area | before | after |
|---|---|---|
| Official API | Not supported by rustifi action registry | 78 captured official operations dispatch through registry metadata |
| Internal inventory | Unverified rows could appear callable | Only verified internal rows are exposed as runtime capabilities |
| Hybrid aliases | Official-first and required `siteId` too often | Internal by default, official with `siteId` or explicit preference |
| Mutations | Extra boolean gate in action params | MCP mounted auth requires `unifi:admin`; no extra action parameter |
| CLI parser | Old tests duplicated parser logic | Tests call real parser and cover generated actions |

## Verification Evidence

| command | expected | actual | status |
|---|---|---|---|
| `cargo fmt --check` | clean formatting | clean after `cargo fmt` | pass |
| `cargo clippy --all-targets -- -D warnings` | no warnings | passed | pass |
| `cargo test` | unit/integration tests pass | passed | pass |
| no-gate string sweep | no removed mutation-gate strings | no matches | pass |
| LOC check | no Rust file over 500 LOC | no output | pass |
| `mod.rs` check | no `mod.rs` files | no output | pass |

## Risks and Rollback

- Risk: generic official dispatch may still need endpoint-specific body schemas. Roll back by reverting this PR or narrowing `official_*` exposure in `src/capabilities/official_network.rs`.
- Risk: unverified internal rows are not callable yet. The rollback path is to keep verified filtering and continue `rustifi-4wo`.

## Decisions Not Taken

- Did not expose unverified internal rows as callable actions.
- Did not add a Python extraction script; `xtask` owns repo-local inventory refresh.
- Did not add endpoint-specific official request schemas in this pass.

## References

- PR: https://github.com/jmagar/unifi-rmcp/pull/2
- Bead: `rustifi-4wo`
- Official inventory: `data/unifi_official_network_v10_3_58.json`
- Internal reference inventory: `data/unifi_internal_reference_tools.json`

## Open Questions

- Which internal reference rows map to verified live UniFi Network V1/V2 endpoints?
- Should mutating MCP calls later use elicitation prompts for interactive clients?

## Next Steps

1. Commit and push the remaining code/docs cleanup.
2. Run a live official smoke test against the Cloud Gateway Max when credentials and `UNIFI_SITE_ID` are available.
3. Work `rustifi-4wo` to verify and safely expand internal runtime capabilities.
