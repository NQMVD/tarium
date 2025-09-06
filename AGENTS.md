
# Tarium — Agents & Project Status

This document is an at-a-glance reference for contributors and automated agents (CI, bots, docs generators) about the Tarium project and its current state during the port to support SPT (Single Player Tarkov).

## Project summary

- Repository: a Rust-based CLI application and libraries for managing game mods and modpacks.
- Primary crates/folders:
  - `src/` — CLI front-end and command wiring for the main binary (to be renamed to `tarium`).
  - `libarov/` — primary library used for the SPT/Tarium port (already named `libarov`).
  - `libium/` — a reference library containing Minecraft-focused logic (kept for reference and comparison only).
  - `SPT/` and `user/` — example/local assets and plugins targeted at SPT mod installations.

## Goal: become a mod manager for SPT (Single Player Tarkov)

The project is being adapted into `tarium`, a mod manager for Single Player Tarkov (SPT). The port focuses on:

- Mapping mod metadata and install flows to SPT's file layout and plugin conventions.
- Supporting SPT-specific mod loaders, profile handling, and installer behaviors.
- Providing a robust CLI for listing, adding, upgrading, and switching profiles targeted at SPT installs.

Note: SPT-specific assets and example mods live in the `SPT/` directory and `test_mods/`.

## Current status (high-level)

- CLI and core behaviors (listing, adding, upgrading) are implemented across the executable and libraries.
- `libarov` is the focal library for the SPT port; `libium` remains in the repo as a reference implementation for Minecraft behavior and APIs.
- Filtering and selection logic (such as `GameVersionStrict`, `GameVersionMinor`, and `select_latest`) exists in the codebase; some implementation details differ between `libium` (fully wired to Modrinth tag data) and `libarov` (mirrors structure).
- Example assets and test mods are present under `test_mods/` and `SPT/` for manual testing and development.

## Architecture notes for agents

- Filters accept an iterator of candidate metadata and return sets of indices that pass. The selection function intersects these sets and picks a chosen index from the remaining candidates.
- Version groups (for `GameVersionMinor`) are cached with `OnceLock`. Agents that update external tag data should account for this caching behavior.
- theres attributes that hide warnings in dev mode, `#[cfg_attr(debug_assertions, allow(dead_code))]`
- for dev purposes the config dir is set to tarium-dev, for realease its just tarium

This file is a concise, living summary for automated agents operating on the repository; update it as the Tarium port progresses.
