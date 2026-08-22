# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`human-emulator` — a Rust simulation of a "healthy/normal" (健常な) human, minute by minute, that
scores how normal the resulting life was. It ships as two front-ends over one library: a native CLI
and a Yew/WebAssembly page deployed to Cloudflare Pages. Edition 2024; the simulation itself is
std-only.

## Commands

```bash
cargo run -- -s 42 -d 7 -t          # CLI: seed 42, 7 days, minute-level timeline
cargo test                          # 35 unit/behaviour tests (all live in the lib)
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --bin web -- -D warnings
cargo fmt
trunk serve                         # dev server on :8080
trunk build --release               # static site into dist/
```

CI runs exactly those checks — see [.github/workflows/deploy.yml](.github/workflows/deploy.yml).

## Layout

- [src/lib.rs](src/lib.rs) — the simulation, shared by both binaries. No I/O, no clock, no OS
  randomness, so it compiles to `wasm32-unknown-unknown` untouched. **Keep it that way**:
  `SystemTime::now()` panics on `wasm32-unknown-unknown`, so the seed must come from the caller.
  - [rng.rs](src/rng.rs) — xorshift64\*. The whole simulation is deterministic in the seed.
  - [clock.rs](src/clock.rs) — minute-resolution clock from Monday 00:00.
  - [needs.rs](src/needs.rs) — five drives on 0–100 with per-minute drift; `mood()` weighs them.
  - [activity.rs](src/activity.rs) — 13 activities × `effect()` (per-minute drive deltas),
    `appropriateness()` (0–1 by hour — **this is where "健常" lives**), `appeal()` (need pressure,
    with a 30-point deadband and a relief cap), `satiation_minutes()` (daily boredom).
  - [human.rs](src/human.rs) — the loop. `obligation()` imposes the weekday commute/work/lunch
    schedule, free time is scored, activities run in blocks truncated at the next boundary, and
    `close_day()` applies the penalties that make up 健常度.
  - [dialogue.rs](src/dialogue.rs) — canned remarks per activity/mood.
- [src/main.rs](src/main.rs) — CLI binary `human-emulator` (default bin). Owns arg parsing and the
  only use of `SystemTime`.
- [src/bin/web.rs](src/bin/web.rs) — Yew app, bin `web`. Everything is inside
  `#[cfg(target_arch = "wasm32")] mod app` with a native `main` that just errors out, so
  `cargo build`/`cargo test` on the host stay clean. Do not add submodule files under `src/bin/`:
  cargo would auto-discover them as extra binaries.
- [index.html](index.html) / [style.css](style.css) / [Trunk.toml](Trunk.toml) / [_headers](_headers)
  — the trunk build. `data-bin="web"` picks the wasm binary.

Browser dependencies (`yew`, `js-sys`, `web-sys`) live under
`[target.'cfg(target_arch = "wasm32")'.dependencies]`, so native builds pull in nothing.

## Conventions

- Comments, test-fn names, and all user-visible output are Japanese; public API names are English.
- Non-ASCII test fn names cannot sit directly after an ASCII word (`fn stamp は…` fails to parse) —
  write `fn stamp_は…`.
- In `html!`, the `for` shorthand only works inside an element, so nested lists need a `<>…</>`.
- Yew 0.23: the macro is `#[component(Name)]` (`function_component` still exists as an alias).
- Tuning the model means changing numbers in `effect` / `appropriateness` / `satiation_minutes`.
  The guard rails are the behaviour tests in `human.rs`: sleeps every night, works every weekday,
  健常度 ≥ 70 across several seeds, no drive pinned at 100, never gets up at 3am, gapless log.
  Past regressions those tests caught: bathing four times in a row, snacking every five minutes,
  waking at 05:00 because a sleep block happened to end there.
