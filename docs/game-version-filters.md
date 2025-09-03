# Game version filters (GameVersionStrict / GameVersionMinor)

This document explains how the game-version related filters work in this repository, where the implementation is, and a few edge cases and suggestions.

## Summary

- `GameVersionStrict(Vec<String>)` — matches files whose metadata `game_versions` contains any of the provided version strings (exact string equality).
- `GameVersionMinor(Vec<String>)` — expands each provided version into a "minor-compatible" group (groups are determined from Modrinth game-version tags where a `major` flag splits groups), then matches any file whose `game_versions` contains any value from those expanded groups.

## Where the code lives

- Filter enums and helpers:
  - `libium/src/config/filters.rs`
  - `libarov/src/config/filters.rs`
- Filter application / selection logic:
  - `libium/src/upgrade/check.rs` (full implementation; fetches Modrinth tag list)
  - `libarov/src/upgrade/check.rs` (grouping currently stubbed/TODO)
- Call sites / consumers:
  - CLI: `src/cli.rs` (builds and pushes filters)
  - Config structs: `libium/src/config/structs.rs`, `libarov/src/config/structs.rs`
  - Add flow: `libium/src/add.rs`
- Utility: `crate::iter_ext::IterExt` (used for collecting matching indices)

## Exact matching rules

- Matching is done using `Vec<String>::contains` on metadata `game_versions` — this is an exact, case-sensitive string match.
- No normalization (semantic version parsing, trimming, or case folding) is applied by the filter code.

## Version grouping (GameVersionMinor)

- `libium` builds groups by fetching Modrinth's game-version tags and grouping release-type tags into buckets split when a tag has `major = true`.
- `libarov` currently has a TODO and sets an empty group; as a result `GameVersionMinor` will not expand groups there until implemented.
- The groups are cached in a `OnceLock` to avoid repeated remote lookups.

## Errors and edge cases

- If a filter yields no matching candidates, `select_latest` returns `Error::FilterEmpty` (listing the empty filters).
- If intersections result in no candidate, `select_latest` returns `IntersectFailure`.
- Case and format mismatches (e.g. `"1.19"` vs `"1.19.2"`) will prevent matches.
- If `GameVersionMinor` grouping yields an empty expansion (e.g. stubbed or unknown tag), it will match nothing by default.

## Suggestions / low-risk improvements

- Implement `get_version_groups()` in `libarov` similar to `libium` or provide a static fallback.
- Normalize version strings (trim, optional canonicalization) before matching to reduce format mismatches.
- Consider falling back to strict matching when `GameVersionMinor` expansion is empty instead of producing zero results.
- Add unit tests for strict/minor matching and for `select_latest` behavior (including ModLoaderPrefer interaction).

## Quick examples

- Metadata: `file.game_versions = ["1.20", "1.20.1"]`
  - `GameVersionStrict(["1.20"])` — match (exact contains).
  - `GameVersionMinor(["1.20"])` — if the group for 1.20 includes `1.20.1`, it will match both.

## Testing tips

- Run unit tests focused on `upgrade::check` and create small `Metadata` vectors to verify the index sets produced by each filter.
- For `libium`, ensure network-backed tests mock or stub Modrinth API calls.

---

If you want this doc expanded into a short repo README section or automated tests added, tell me which next.
