//! human-emulator — 健常な人間のシミュレータ。
//!
//! 欲求(眠気・空腹・寂しさ・不潔感・ストレス)が時間とともに溜まり、
//! 社会的スケジュールがそれを律する。出てくる一週間がどれだけ
//! 「ふつう」だったかを健常度として採点する。

mod activity;
mod clock;
mod dialogue;
mod human;
mod needs;
mod rng;

use activity::Activity;
use human::Human;
use needs::Drive;
use std::time::{SystemTime, UNIX_EPOCH};

struct Options {
    name: String,
    days: u32,
    seed: u64,
    timeline: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            name: "被験者".to_string(),
            days: 7,
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x5EED),
            timeline: false,
        }
    }
}

const USAGE: &str = "\
human-emulator — 健常な人間エミュレータ

使い方:
    human-emulator [オプション]

オプション:
    -d, --days <N>     シミュレートする日数 (既定: 7)
    -s, --seed <N>     乱数シード (省略時は現在時刻)
    -n, --name <名前>  被験者の名前 (既定: 被験者)
    -t, --timeline     一日の行動を分単位のログで表示する
    -h, --help         この説明を表示する
";

fn parse_args() -> Result<Options, String> {
    let mut opts = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| -> Result<String, String> {
            args.next().ok_or_else(|| format!("{flag} に値がない"))
        };
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-d" | "--days" => {
                opts.days = value("--days")?
                    .parse()
                    .map_err(|_| "--days は正の整数で指定する".to_string())?;
            }
            "-s" | "--seed" => {
                opts.seed = value("--seed")?
                    .parse()
                    .map_err(|_| "--seed は整数で指定する".to_string())?;
            }
            "-n" | "--name" => opts.name = value("--name")?,
            "-t" | "--timeline" => opts.timeline = true,
            other => return Err(format!("知らないオプション: {other}")),
        }
    }
    if opts.days == 0 {
        return Err("--days は 1 以上にする".to_string());
    }
    Ok(opts)
}

fn main() {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("エラー: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    let mut human = Human::new(opts.name.clone(), opts.seed);
    human.run(opts.days);

    println!("=== human-emulator ===");
    println!(
        "被験者: {}  /  {} 日間  /  seed {}",
        human.name, opts.days, opts.seed
    );
    println!();

    if opts.timeline {
        print_timeline(&human);
    }
    print_days(&human);
    print_summary(&human);
}

/// 境界で切れただけの同じ行動は、一つの塊として見せる。
fn merged(human: &Human) -> Vec<human::Entry> {
    let mut out: Vec<human::Entry> = Vec::new();
    for e in &human.log {
        match out.last_mut() {
            Some(prev) if prev.activity == e.activity && prev.at.day() == e.at.day() => {
                prev.minutes += e.minutes;
                prev.mood = e.mood;
                prev.remark = e.remark;
            }
            _ => out.push(e.clone()),
        }
    }
    out
}

fn print_timeline(human: &Human) {
    let mut current_day = u32::MAX;
    for e in &merged(human) {
        if e.at.day() != current_day {
            current_day = e.at.day();
            println!("── {}日目({}) ──", current_day + 1, e.at.weekday().label());
        }
        println!(
            "  {}  {:<10} {:>3}分  気分{:>3.0}  {}",
            e.at.hhmm(),
            e.activity.label(),
            e.minutes,
            e.mood,
            e.remark
        );
    }
    println!();
}

fn print_days(human: &Human) {
    println!("── 日報 ──");
    for d in &human.days {
        println!(
            "{}日目({})  睡眠{:>4}分  勤務{:>4}分  対人{:>3}分  食事{}回   健常度 {:>5.1} [{}]",
            d.day + 1,
            d.weekday.label(),
            d.sleep_minutes,
            d.work_minutes,
            d.social_minutes,
            d.meals,
            d.score,
            d.verdict()
        );
        for (reason, cost) in &d.penalties {
            println!("            - {reason} (-{cost:.0})");
        }
    }
    println!();
}

fn print_summary(human: &Human) {
    let total: u32 = human.log.iter().map(|e| e.minutes).sum();
    println!("── 総括 ──");
    println!("終了時刻 : {}", human.clock.stamp());
    println!("平均気分 : {:.1} / 100", human.average_mood());
    println!(
        "健常度   : {:.1} / 100  [{}]",
        human.normalcy(),
        human.verdict()
    );
    println!();

    println!("時間の使い方:");
    let mut rows: Vec<(Activity, u32)> = Activity::VOLUNTARY
        .iter()
        .copied()
        .chain([Activity::Work, Activity::Commute])
        .map(|a| {
            let m = human
                .log
                .iter()
                .filter(|e| e.activity == a)
                .map(|e| e.minutes)
                .sum();
            (a, m)
        })
        .filter(|(_, m)| *m > 0)
        .collect();
    rows.sort_by_key(|(_, m)| std::cmp::Reverse(*m));
    for (a, m) in rows {
        let share = m as f32 / total.max(1) as f32;
        let bar = "█".repeat(((share * 40.0).round() as usize).max(1));
        println!(
            "  {:<10} {:>5}分 ({:>4.1}%) {}",
            a.label(),
            m,
            share * 100.0,
            bar
        );
    }
    println!();

    println!("最終状態の欲求:");
    for d in Drive::ALL {
        let v = human.needs.get(d);
        let filled = (v / 5.0).round() as usize;
        println!(
            "  {:<8} {:>5.1}  {}{}",
            d.label(),
            v,
            "▓".repeat(filled),
            "░".repeat(20usize.saturating_sub(filled))
        );
    }
    println!(
        "  気分     {:>5.1}  ({})",
        human.needs.mood(),
        human.needs.mood_label()
    );
    let (drive, level) = human.needs.most_urgent();
    println!(
        "  いま最も逼迫しているのは「{}」({:.1})",
        drive.label(),
        level
    );
}
