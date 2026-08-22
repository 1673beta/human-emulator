# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`human-emulator` — a dependency-free Rust binary that simulates a "healthy/normal" (健常な) human
week minute by minute and scores how normal the resulting behaviour was. Edition 2024, std only,
no `Cargo.lock` committed, `/target` gitignored.

## Commands

```bash
cargo run                       # 7 simulated days, random seed
cargo run -- -s 42 -d 7 -t      # fixed seed, 7 days, minute-level timeline
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt
```

CLI flags: `--days/-d`, `--seed/-s`, `--name/-n`, `--timeline/-t`, `--help/-h`.

## Architecture

Everything is modules of the binary crate (`src/main.rs` declares them; no `lib.rs`).

- [rng.rs](src/rng.rs) — xorshift64\* PRNG. The whole simulation is deterministic in the seed;
  the test `同じシードは同じ一週間を生む` depends on that.
- [clock.rs](src/clock.rs) — minute-resolution clock starting Monday 00:00, weekday/weekend.
- [needs.rs](src/needs.rs) — five drives (眠気/空腹/寂しさ/不潔感/ストレス) on 0–100, each with a
  per-minute base drift; `mood()` is a weighted complement of them.
- [activity.rs](src/activity.rs) — the 13 activities. Three knobs drive all behaviour:
  `effect()` (per-minute drive deltas), `appropriateness()` (0–1, how normal that activity is at
  that hour — **this is where "健常" actually lives**), and `appeal()` (need pressure, with a
  30-point deadband and a relief cap so fast-satisfying acts don't always win).
- [human.rs](src/human.rs) — the loop. `obligation()` imposes the weekday commute/work/lunch
  schedule; free time is chosen by `appeal × leisure bonus × satiation × appropriateness × jitter`.
  Activities run in blocks truncated at the next `BOUNDARIES` time. `close_day()` scores each day
  by penalties (睡眠不足, 欠食, 孤立, 入浴なし, 勤務不足, 夜更かし, 気分の落ち込み).
- [dialogue.rs](src/dialogue.rs) — canned remarks per activity/mood.

## Conventions

- Comments, identifiers in tests, and all output are Japanese; public API names are English.
- Non-ASCII test fn names cannot sit directly after an ASCII word (`fn stamp は…` fails to parse) —
  use `fn stamp_は…`.
- Tuning the model means changing numbers in `effect`/`appropriateness`/`satiation_minutes`. The
  guard rails are the behavioural tests in `human.rs`: sleeps every night, works every weekday,
  健常度 ≥ 70 across several seeds, no drive pinned at 100.
