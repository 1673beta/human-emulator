//! 健常な人間のシミュレーション本体。
//!
//! ネイティブ CLI ([`bin/cli`](../src/main.rs)) と Yew/WebAssembly のフロントエンドが
//! このライブラリを共有する。std のみに依存し、時刻・乱数源・I/O を一切持たないので
//! `wasm32-unknown-unknown` でもそのまま動く。

pub mod activity;
pub mod clock;
pub mod dialogue;
pub mod human;
pub mod needs;
pub mod rng;

pub use activity::Activity;
pub use clock::{Clock, Weekday};
pub use human::{DayReport, Entry, Human};
pub use needs::{Drive, Needs};
pub use rng::Rng;
