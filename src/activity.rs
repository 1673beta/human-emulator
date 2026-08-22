//! 人間がとりうる行動と、その行動が欲求に与える 1 分あたりの効果。

use crate::clock::Clock;
use crate::needs::Drive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Sleep,
    Nap,
    Meal,
    Snack,
    Bathe,
    Commute,
    Work,
    Chat,
    Meetup,
    Exercise,
    Chores,
    Hobby,
    Idle,
}

impl Activity {
    /// 添字づけのための全行動。
    pub const ALL: [Activity; 13] = [
        Activity::Sleep,
        Activity::Nap,
        Activity::Meal,
        Activity::Snack,
        Activity::Bathe,
        Activity::Commute,
        Activity::Work,
        Activity::Chat,
        Activity::Meetup,
        Activity::Exercise,
        Activity::Chores,
        Activity::Hobby,
        Activity::Idle,
    ];

    /// 自由時間に自発的に選ばれうる行動。義務(通勤・勤務)は含まない。
    pub const VOLUNTARY: [Activity; 11] = [
        Activity::Sleep,
        Activity::Nap,
        Activity::Meal,
        Activity::Snack,
        Activity::Bathe,
        Activity::Chat,
        Activity::Meetup,
        Activity::Exercise,
        Activity::Chores,
        Activity::Hobby,
        Activity::Idle,
    ];

    pub fn index(self) -> usize {
        match self {
            Activity::Sleep => 0,
            Activity::Nap => 1,
            Activity::Meal => 2,
            Activity::Snack => 3,
            Activity::Bathe => 4,
            Activity::Commute => 5,
            Activity::Work => 6,
            Activity::Chat => 7,
            Activity::Meetup => 8,
            Activity::Exercise => 9,
            Activity::Chores => 10,
            Activity::Hobby => 11,
            Activity::Idle => 12,
        }
    }

    /// その日にこれだけやると満足する分数。超えるほど魅力が落ちる。
    /// 一日三度の運動も十時間の趣味も、健常の範囲からは外れる。
    pub fn satiation_minutes(self) -> f32 {
        match self {
            Activity::Sleep => 480.0,
            Activity::Nap => 30.0,
            Activity::Meal => 90.0,
            Activity::Snack => 25.0,
            Activity::Bathe => 25.0,
            Activity::Chat => 45.0,
            Activity::Meetup => 180.0,
            Activity::Exercise => 30.0,
            Activity::Chores => 60.0,
            Activity::Hobby => 180.0,
            Activity::Idle => 60.0,
            Activity::Commute | Activity::Work => f32::INFINITY,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Activity::Sleep => "睡眠",
            Activity::Nap => "仮眠",
            Activity::Meal => "食事",
            Activity::Snack => "間食",
            Activity::Bathe => "入浴",
            Activity::Commute => "移動",
            Activity::Work => "仕事",
            Activity::Chat => "連絡・雑談",
            Activity::Meetup => "人と会う",
            Activity::Exercise => "運動",
            Activity::Chores => "家事",
            Activity::Hobby => "趣味",
            Activity::Idle => "ぼんやり",
        }
    }

    /// 標準的な所要時間の範囲(分)。実際の値はここから乱択される。
    pub fn duration_range(self) -> (u32, u32) {
        match self {
            Activity::Sleep => (390, 510), // 6.5〜8.5 時間
            Activity::Nap => (20, 40),
            Activity::Meal => (25, 45),
            Activity::Snack => (5, 15),
            Activity::Bathe => (15, 30),
            Activity::Commute => (30, 60),
            Activity::Work => (60, 60), // 1 時間刻みで再判断する
            Activity::Chat => (10, 30),
            Activity::Meetup => (90, 180),
            Activity::Exercise => (30, 60),
            Activity::Chores => (20, 50),
            Activity::Hobby => (45, 120),
            Activity::Idle => (10, 30),
        }
    }

    /// 1 分あたりの欲求の増減。負が「満たす」方向。
    pub fn effect(self, d: Drive) -> f32 {
        use Activity::*;
        use Drive::*;
        match (self, d) {
            // 睡眠中は代謝が落ちるので、放置分の欲求上昇を打ち消す向きに効く。
            (Sleep, Sleepiness) => -0.55,
            (Sleep, Hunger) => -0.23,
            (Sleep, Stress) => -0.06,
            (Sleep, Loneliness) => -0.04,

            (Nap, Sleepiness) => -0.60,
            (Nap, Hunger) => -0.20,

            (Meal, Hunger) => -3.0,
            (Meal, Stress) => -0.10,
            (Meal, Grime) => 0.05,

            (Snack, Hunger) => -2.0,
            (Snack, Stress) => -0.05,

            (Bathe, Grime) => -6.0,
            (Bathe, Stress) => -0.30,
            (Bathe, Sleepiness) => -0.10,

            (Commute, Stress) => 0.05,
            (Commute, Sleepiness) => 0.03,
            (Commute, Grime) => 0.04,
            (Commute, Hunger) => 0.10,

            (Work, Stress) => 0.07,
            (Work, Hunger) => 0.15,
            (Work, Grime) => 0.05,
            (Work, Loneliness) => -0.04,

            (Chat, Loneliness) => -1.50,
            (Chat, Stress) => -0.10,

            (Meetup, Loneliness) => -1.00,
            (Meetup, Stress) => -0.20,
            (Meetup, Hunger) => 0.20,
            (Meetup, Grime) => 0.05,

            (Exercise, Stress) => -0.50,
            (Exercise, Grime) => 1.00,
            (Exercise, Hunger) => 0.40,
            (Exercise, Sleepiness) => 0.15,

            (Chores, Grime) => -1.20,
            (Chores, Stress) => 0.03,

            (Hobby, Stress) => -0.45,
            (Hobby, Loneliness) => -0.10,

            (Idle, Stress) => -0.15,
            (Idle, Sleepiness) => 0.05,

            _ => 0.0,
        }
    }

    /// その時刻にその行動をとるのが「ふつう」かどうか。0.0〜1.0。
    /// 健常さの本体はここにある — 欲求ではなく時間帯が行動を律する。
    pub fn appropriateness(self, clock: Clock) -> f32 {
        let h = clock.hour();
        let weekend = clock.weekday().is_weekend();
        match self {
            Activity::Sleep => match h {
                23 | 0..=5 => 1.0,
                22 => 0.8,
                6 => 0.4,
                7 => {
                    if weekend {
                        0.7
                    } else {
                        0.03
                    }
                }
                8 => {
                    if weekend {
                        0.35
                    } else {
                        0.02
                    }
                }
                _ => 0.02,
            },
            Activity::Nap => match h {
                13..=15 => {
                    if weekend {
                        0.7
                    } else {
                        0.3
                    }
                }
                _ => 0.05,
            },
            Activity::Meal => match h {
                7..=8 | 12..=13 | 18..=20 => 1.0,
                9..=11 | 14..=17 | 21 => 0.25,
                _ => 0.03,
            },
            Activity::Snack => match h {
                10..=11 | 15..=16 | 21..=22 => 0.8,
                7..=9 | 12..=14 | 17..=20 => 0.3,
                _ => 0.05,
            },
            Activity::Bathe => match h {
                6..=8 | 19..=23 => 1.0,
                9..=18 => 0.2,
                _ => 0.05,
            },
            Activity::Commute | Activity::Work => 0.0, // 義務側で管理する
            Activity::Chat => match h {
                8..=22 => 1.0,
                7 | 23 => 0.3,
                _ => 0.02,
            },
            Activity::Meetup => match h {
                11..=21 if weekend => 1.0,
                19..=21 => 0.7,
                11..=18 => 0.15,
                _ => 0.02,
            },
            Activity::Exercise => match h {
                6..=8 | 17..=20 => 1.0,
                9..=16 => {
                    if weekend {
                        0.8
                    } else {
                        0.2
                    }
                }
                _ => 0.05,
            },
            Activity::Chores => match h {
                9..=11 if weekend => 1.0,
                7..=21 => 0.6,
                _ => 0.05,
            },
            Activity::Hobby => match h {
                19..=22 => 1.0,
                9..=18 => {
                    if weekend {
                        0.9
                    } else {
                        0.3
                    }
                }
                23 => 0.3,
                _ => 0.05,
            },
            Activity::Idle => 0.4,
        }
    }

    /// 用事ではなく暇つぶしとして選ばれる行動か。
    /// 欲求が満たされていても、健常な人間は空いた時間に何かをする。
    pub fn is_leisure(self) -> bool {
        matches!(
            self,
            Activity::Hobby | Activity::Chores | Activity::Idle | Activity::Exercise
        )
    }

    /// 「今これをやりたい」の強さ。逼迫した欲求ほど二乗で効く。
    pub fn appeal(self, needs: &crate::needs::Needs) -> f32 {
        let mut score = 0.0;
        for d in Drive::ALL {
            // 解消の速さには上限を置く。置かないと、一瞬で満たせる行動
            // (食事・入浴)が僅かな欲求でも常に勝ってしまう。
            let relief = (-self.effect(d)).min(2.0);
            if relief > 0.0 {
                // 3 割に満たない欲求は行動の動機にならない(不感帯)。
                let pressure = ((needs.get(d) - 30.0) / 70.0).max(0.0);
                score += relief * pressure * pressure * 10.0;
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::MINUTES_PER_DAY;
    use crate::needs::Needs;

    fn 時刻(day: u32, hour: u32) -> Clock {
        Clock {
            elapsed: day * MINUTES_PER_DAY + hour * 60,
        }
    }

    #[test]
    fn 深夜の睡眠は適切で真昼の睡眠は不適切() {
        assert!(Activity::Sleep.appropriateness(時刻(0, 2)) > 0.9);
        assert!(Activity::Sleep.appropriateness(時刻(0, 14)) < 0.1);
    }

    #[test]
    fn 食事は食事時に寄る() {
        assert!(
            Activity::Meal.appropriateness(時刻(0, 12))
                > Activity::Meal.appropriateness(時刻(0, 3))
        );
    }

    #[test]
    fn 人と会うのは週末の方が起きやすい() {
        let 平日 = Activity::Meetup.appropriateness(時刻(0, 14)); // 月
        let 週末 = Activity::Meetup.appropriateness(時刻(5, 14)); // 土
        assert!(週末 > 平日);
    }

    #[test]
    fn 空腹なら食事の魅力が上がる() {
        let mut n = Needs::rested();
        n.set(Drive::Hunger, 10.0);
        let 満腹時 = Activity::Meal.appeal(&n);
        n.set(Drive::Hunger, 95.0);
        assert!(Activity::Meal.appeal(&n) > 満腹時);
    }

    #[test]
    fn 睡眠は眠気を解消する方向に効く() {
        assert!(Activity::Sleep.effect(Drive::Sleepiness) < 0.0);
        assert!(Activity::Bathe.effect(Drive::Grime) < 0.0);
        assert!(Activity::Work.effect(Drive::Stress) > 0.0);
    }

    #[test]
    fn 所要時間の範囲は妥当() {
        for a in Activity::VOLUNTARY {
            let (lo, hi) = a.duration_range();
            assert!(lo > 0 && lo <= hi, "{:?} の範囲が不正", a);
        }
    }
}
