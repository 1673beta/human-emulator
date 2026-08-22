//! エミュレータ本体。欲求・時計・社会的スケジュールを噛み合わせて
//! 一人の「健常な人間」の一日を回す。

use crate::activity::Activity;
use crate::clock::{Clock, MINUTES_PER_DAY, Weekday};
use crate::dialogue;
use crate::needs::{Drive, Needs};
use crate::rng::Rng;

/// 平日に行動を再検討する時刻(分)。出退勤と起床時刻。
const WEEKDAY_BOUNDARIES: [u32; 7] = [
    7 * 60,       // 起床(目覚まし)
    8 * 60,       // 出発
    9 * 60,       // 始業
    12 * 60 + 30, // 昼休み
    13 * 60 + 15, // 午後の始業
    18 * 60,      // 終業
    19 * 60,      // 帰宅
];

/// 週末は目覚ましだけが予定。
const WEEKEND_BOUNDARIES: [u32; 1] = [9 * 60];

fn boundaries_for(weekday: Weekday) -> &'static [u32] {
    if weekday.is_weekend() {
        &WEEKEND_BOUNDARIES
    } else {
        &WEEKDAY_BOUNDARIES
    }
}

/// 平日に外から課される予定。健常さは意志ではなく予定表が支えている。
fn obligation(clock: Clock) -> Option<Activity> {
    if clock.weekday().is_weekend() {
        return None;
    }
    match clock.minute_of_day() {
        480..540 => Some(Activity::Commute),
        540..750 => Some(Activity::Work),
        750..795 => Some(Activity::Meal),
        795..1080 => Some(Activity::Work),
        1080..1140 => Some(Activity::Commute),
        _ => None,
    }
}

/// 次に行動を見直す時刻(起点からの絶対分)。
fn next_boundary(clock: Clock) -> u32 {
    let now = clock.minute_of_day();
    let midnight = clock.elapsed - now;
    for &b in boundaries_for(clock.weekday()) {
        if b > now {
            return midnight + b;
        }
    }
    let tomorrow = Weekday::from_index(clock.day() + 1);
    midnight + MINUTES_PER_DAY + boundaries_for(tomorrow)[0]
}

/// 概日リズムによる眠気の溜まりやすさ。深夜ほど大きい。
fn circadian(hour: u32) -> f32 {
    match hour {
        0..=5 => 2.0,
        6..=8 => 0.7,
        9..=12 => 0.6,
        13..=15 => 1.0, // 昼食後の落ち込み
        16..=19 => 0.8,
        20..=21 => 1.2,
        _ => 1.7,
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub at: Clock,
    pub activity: Activity,
    pub minutes: u32,
    pub mood: f32,
    pub remark: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
struct DayStats {
    sleep: u32,
    work: u32,
    social: u32,
    bathe: u32,
    meals: u32,
    low_mood: u32,
    late_night: bool,
}

#[derive(Debug, Clone)]
pub struct DayReport {
    pub day: u32,
    pub weekday: Weekday,
    pub sleep_minutes: u32,
    pub work_minutes: u32,
    pub social_minutes: u32,
    pub meals: u32,
    pub penalties: Vec<(&'static str, f32)>,
    pub score: f32,
}

impl DayReport {
    pub fn verdict(&self) -> &'static str {
        match self.score {
            s if s >= 85.0 => "健常",
            s if s >= 70.0 => "おおむね健常",
            s if s >= 50.0 => "要注意",
            _ => "不調",
        }
    }
}

pub struct Human {
    pub name: String,
    pub clock: Clock,
    pub needs: Needs,
    pub log: Vec<Entry>,
    pub days: Vec<DayReport>,
    rng: Rng,
    last: Option<Activity>,
    /// その日ここまでに各行動へ費やした分数(飽きの計算用)。
    spent_today: [u32; Activity::ALL.len()],
    today: DayStats,
    mood_sum: f64,
    mood_minutes: u32,
}

impl Human {
    /// 月曜の朝 6 時、前夜 23 時に就寝した状態から始める。
    pub fn new(name: impl Into<String>, seed: u64) -> Self {
        Self {
            name: name.into(),
            clock: {
                let mut c = Clock::start();
                c.advance(6 * 60);
                c
            },
            needs: Needs::rested(),
            log: Vec::new(),
            days: Vec::new(),
            rng: Rng::new(seed),
            last: None,
            spent_today: [0; Activity::ALL.len()],
            today: DayStats {
                sleep: 7 * 60, // 前夜ぶんを計上しておく
                ..DayStats::default()
            },
            mood_sum: 0.0,
            mood_minutes: 0,
        }
    }

    /// `days` 日ぶん回す。
    pub fn run(&mut self, days: u32) {
        let end = self.clock.elapsed + days * MINUTES_PER_DAY;
        while self.clock.elapsed < end {
            self.step(end);
        }
        // 半日以上残っていれば、最後の未完の一日も評価する。
        if self.clock.minute_of_day() >= 12 * 60 {
            self.close_day();
        }
    }

    fn step(&mut self, end: u32) {
        let at = self.clock;
        let activity = self.choose();
        let remaining = end - at.elapsed;
        let to_boundary = next_boundary(at) - at.elapsed;
        let mut minutes = self
            .duration_for(activity)
            .min(to_boundary)
            .min(remaining)
            .max(1);

        // 眠りが真夜中に切れても人は起き出さない。目覚まし(次の境界)まで寝る。
        if activity == Activity::Sleep {
            let wake_hour = ((at.elapsed + minutes) % MINUTES_PER_DAY) / 60;
            if wake_hour < 6 {
                minutes = to_boundary.min(remaining).max(minutes);
            }
        }

        self.note_session(activity);
        self.apply(activity, minutes);
        self.last = Some(activity);

        let mood = self.needs.mood();
        let remark = dialogue::remark(activity, mood, &mut self.rng);
        self.log.push(Entry {
            at,
            activity,
            minutes,
            mood,
            remark,
        });
    }

    fn choose(&mut self) -> Activity {
        if let Some(a) = obligation(self.clock) {
            return a;
        }
        let mut best = (Activity::Idle, f32::MIN);
        for a in Activity::VOLUNTARY {
            // 欲求の切迫さに、その時間帯としての「ふつうさ」を掛ける。
            let mut score = a.appeal(&self.needs);
            if a.is_leisure() {
                // 用事がなくても空き時間は埋まる。
                score += 1.2;
            }
            // 同じことを一日に何度もやるほど飽きる。
            score *= 1.0 / (1.0 + self.spent_today[a.index()] as f32 / a.satiation_minutes());
            score *= a.appropriateness(self.clock);
            if self.last == Some(a) {
                // 直後の繰り返しは不自然(風呂に四回入る人間はいない)。
                score *= 0.25;
            }
            score *= self.rng.range(0.85, 1.15);
            if score > best.1 {
                best = (a, score);
            }
        }
        best.0
    }

    fn duration_for(&mut self, a: Activity) -> u32 {
        let (lo, hi) = a.duration_range();
        let raw = self.rng.range(lo as f32, hi as f32) as u32;
        (raw / 5).max(1) * 5
    }

    /// 回数で数える指標は行動の開始時に一度だけ計上する。
    fn note_session(&mut self, a: Activity) {
        match a {
            Activity::Meal => self.today.meals += 1,
            Activity::Sleep => {
                let start = self.clock.minute_of_day();
                if (0..5 * 60).contains(&start) {
                    self.today.late_night = true;
                }
            }
            _ => {}
        }
    }

    fn apply(&mut self, a: Activity, minutes: u32) {
        for _ in 0..minutes {
            self.needs.drift(1, circadian(self.clock.hour()));
            for d in Drive::ALL {
                self.needs.adjust(d, a.effect(d));
            }

            let mood = self.needs.mood();
            self.mood_sum += mood as f64;
            self.mood_minutes += 1;
            if mood < 30.0 {
                self.today.low_mood += 1;
            }
            self.spent_today[a.index()] += 1;
            match a {
                Activity::Sleep | Activity::Nap => self.today.sleep += 1,
                Activity::Work => self.today.work += 1,
                Activity::Chat | Activity::Meetup => self.today.social += 1,
                Activity::Bathe => self.today.bathe += 1,
                _ => {}
            }

            self.clock.advance(1);
            if self.clock.minute_of_day() == 0 {
                self.close_day();
            }
        }
    }

    fn close_day(&mut self) {
        // 日付が繰り上がった直後に呼ばれるので、締める対象は前日。
        let day = self
            .clock
            .day()
            .saturating_sub(if self.clock.minute_of_day() == 0 {
                1
            } else {
                0
            });
        let weekday = Weekday::from_index(day);
        let s = self.today;

        let mut penalties: Vec<(&'static str, f32)> = Vec::new();
        if s.sleep < 6 * 60 {
            penalties.push(("睡眠不足", 12.0));
        } else if s.sleep < 7 * 60 {
            penalties.push(("やや寝不足", 5.0));
        }
        if s.meals < 2 {
            penalties.push(("欠食", 8.0));
        } else if s.meals == 2 {
            penalties.push(("食事が二回だけ", 3.0));
        }
        if s.social == 0 {
            penalties.push(("誰とも関わっていない", 6.0));
        }
        if s.bathe == 0 {
            penalties.push(("入浴なし", 6.0));
        }
        if !weekday.is_weekend() && s.work < 5 * 60 {
            penalties.push(("勤務時間の不足", 10.0));
        }
        if s.late_night {
            penalties.push(("夜更かし", 6.0));
        }
        if s.low_mood > 3 * 60 {
            penalties.push(("気分の落ち込みが長い", 5.0));
        }

        let score = (100.0 - penalties.iter().map(|p| p.1).sum::<f32>()).clamp(0.0, 100.0);
        self.days.push(DayReport {
            day,
            weekday,
            sleep_minutes: s.sleep,
            work_minutes: s.work,
            social_minutes: s.social,
            meals: s.meals,
            penalties,
            score,
        });
        self.today = DayStats::default();
        self.spent_today = [0; Activity::ALL.len()];
    }

    pub fn average_mood(&self) -> f32 {
        if self.mood_minutes == 0 {
            return 0.0;
        }
        (self.mood_sum / self.mood_minutes as f64) as f32
    }

    /// 全期間の健常度。
    pub fn normalcy(&self) -> f32 {
        if self.days.is_empty() {
            return 100.0;
        }
        self.days.iter().map(|d| d.score).sum::<f32>() / self.days.len() as f32
    }

    pub fn verdict(&self) -> &'static str {
        match self.normalcy() {
            s if s >= 85.0 => "健常",
            s if s >= 70.0 => "おおむね健常",
            s if s >= 50.0 => "要注意",
            _ => "不調",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn 時刻(day: u32, hour: u32, minute: u32) -> Clock {
        Clock {
            elapsed: day * MINUTES_PER_DAY + hour * 60 + minute,
        }
    }

    #[test]
    fn 平日の日中は仕事が課される() {
        assert_eq!(obligation(時刻(0, 10, 0)), Some(Activity::Work));
        assert_eq!(obligation(時刻(0, 8, 30)), Some(Activity::Commute));
        assert_eq!(obligation(時刻(0, 12, 45)), Some(Activity::Meal));
    }

    #[test]
    fn 週末に義務はない() {
        assert_eq!(obligation(時刻(5, 10, 0)), None);
        assert_eq!(obligation(時刻(6, 14, 0)), None);
    }

    #[test]
    fn 夜間に義務はない() {
        assert_eq!(obligation(時刻(0, 3, 0)), None);
        assert_eq!(obligation(時刻(0, 22, 0)), None);
    }

    #[test]
    fn 境界は必ず前に進む() {
        for day in 0..7 {
            for m in 0..MINUTES_PER_DAY {
                let c = Clock {
                    elapsed: day * MINUTES_PER_DAY + m,
                };
                assert!(next_boundary(c) > c.elapsed, "{day}日目 {m}分 で停滞した");
            }
        }
    }

    #[test]
    fn 週末は目覚ましだけが境界になる() {
        let 土曜昼 = 時刻(5, 12, 0);
        assert_eq!(next_boundary(土曜昼), 時刻(6, 9, 0).elapsed);
    }

    #[test]
    fn 一週間回しても時計が壊れない() {
        let mut h = Human::new("被験者", 12345);
        h.run(7);
        assert_eq!(h.clock.elapsed, 6 * 60 + 7 * MINUTES_PER_DAY);
        assert!(!h.log.is_empty());
    }

    #[test]
    fn 記録は連続していて隙間がない() {
        let mut h = Human::new("被験者", 999);
        h.run(3);
        let mut cursor = 6 * 60;
        for e in &h.log {
            assert_eq!(e.at.elapsed, cursor, "{} で不連続", e.at.stamp());
            cursor += e.minutes;
        }
        assert_eq!(cursor, h.clock.elapsed);
    }

    #[test]
    fn 健常な人間は毎晩眠り毎日働く() {
        let mut h = Human::new("被験者", 2024);
        h.run(7);
        for d in &h.days {
            assert!(
                d.sleep_minutes >= 5 * 60,
                "{}日目の睡眠が {} 分",
                d.day + 1,
                d.sleep_minutes
            );
            if !d.weekday.is_weekend() {
                assert!(
                    d.work_minutes >= 6 * 60,
                    "{}日目の勤務が {} 分",
                    d.day + 1,
                    d.work_minutes
                );
            }
        }
    }

    #[test]
    fn 健常度は高く保たれる() {
        for seed in [1u64, 7, 42, 1000, 65535] {
            let mut h = Human::new("被験者", seed);
            h.run(7);
            assert!(
                h.normalcy() >= 70.0,
                "seed {seed} の健常度が {}",
                h.normalcy()
            );
            assert!(h.average_mood() > 40.0, "seed {seed} の平均気分が低すぎる");
        }
    }

    #[test]
    fn 真夜中に起き出さない() {
        let mut h = Human::new("被験者", 7);
        h.run(14);
        for e in &h.log {
            let hour = e.at.hour();
            if (1..5).contains(&hour) {
                assert_eq!(
                    e.activity,
                    Activity::Sleep,
                    "{} に{}を始めている",
                    e.at.stamp(),
                    e.activity.label()
                );
            }
        }
    }

    #[test]
    fn 欲求が振り切れたままにならない() {
        let mut h = Human::new("被験者", 88);
        h.run(5);
        for d in Drive::ALL {
            assert!(h.needs.get(d) < 100.0, "{} が振り切れている", d.label());
        }
    }

    #[test]
    fn 同じシードは同じ一週間を生む() {
        let mut a = Human::new("A", 4649);
        let mut b = Human::new("B", 4649);
        a.run(7);
        b.run(7);
        assert_eq!(a.log.len(), b.log.len());
        for (x, y) in a.log.iter().zip(b.log.iter()) {
            assert_eq!(x.activity, y.activity);
            assert_eq!(x.minutes, y.minutes);
        }
    }

    #[test]
    fn 日報は日数ぶん出る() {
        let mut h = Human::new("被験者", 5);
        h.run(7);
        assert_eq!(h.days.len(), 7);
    }
}
