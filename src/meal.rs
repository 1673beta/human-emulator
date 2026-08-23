//! 食事の時間帯と献立。
//!
//! 健常な人間は「腹が減ったから食べる」のではなく「食事の時刻だから食べる」。
//! だから食事は一日三つの枠(朝・昼・夕)に紐づき、枠はそれぞれ一度しか使えない。
//! 何を食べたかは疲れ具合で決まり、その中身が翌日の評価に効く。

use crate::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Course {
    Breakfast,
    Lunch,
    Dinner,
}

impl Course {
    pub const ALL: [Course; 3] = [Course::Breakfast, Course::Lunch, Course::Dinner];

    pub fn label(self) -> &'static str {
        match self {
            Course::Breakfast => "朝食",
            Course::Lunch => "昼食",
            Course::Dinner => "夕食",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Course::Breakfast => 0,
            Course::Lunch => 1,
            Course::Dinner => 2,
        }
    }

    /// その時刻がどの食事の枠か。枠の外では食事をとらない。
    pub fn from_hour(hour: u32) -> Option<Course> {
        match hour {
            6..=9 => Some(Course::Breakfast),
            11..=14 => Some(Course::Lunch),
            17..=21 => Some(Course::Dinner),
            _ => None,
        }
    }

    /// 枠の中でも「ふつうの時刻」からずれるほど下がる。
    pub fn appropriateness(hour: u32) -> f32 {
        match hour {
            7..=8 | 12..=13 | 18..=20 => 1.0,
            6 | 9 | 11 | 14 | 17 | 21 => 0.6,
            _ => 0.0,
        }
    }
}

/// 一品ではなく一食ぶんの献立。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dish {
    pub name: &'static str,
    /// 栄養の充実度 0〜100。日ごとの平均が健常度に効く。
    pub nutrition: f32,
    /// 野菜が入っているか。
    pub vegetables: bool,
    /// 満腹度 0〜1。低いと空腹を残すので、早い時間に間食したくなる。
    pub satiety: f32,
}

impl Dish {
    /// この献立で下げられる空腹の下限。満腹度 1.0 なら 0 まで下がる。
    pub fn hunger_floor(self) -> f32 {
        100.0 * (1.0 - self.satiety)
    }
}

const fn dish(name: &'static str, nutrition: f32, vegetables: bool, satiety: f32) -> Dish {
    Dish {
        name,
        nutrition,
        vegetables,
        satiety,
    }
}

/// 献立は栄養の低い順に並べる。手を抜くほど前の方が選ばれる。
const BREAKFAST: [Dish; 5] = [
    dish("コーヒーだけ", 5.0, false, 0.25),
    dish("菓子パン", 25.0, false, 0.7),
    dish("グラノーラとヨーグルト", 60.0, false, 0.8),
    dish("トーストと目玉焼きとサラダ", 75.0, true, 0.9),
    dish("ごはんと味噌汁と焼き鮭", 90.0, true, 1.0),
];

const LUNCH: [Dish; 5] = [
    dish("カップ麺", 20.0, false, 0.7),
    dish("菓子パンとコーヒー", 30.0, false, 0.7),
    dish("コンビニ弁当", 50.0, false, 0.95),
    dish("サンドイッチとサラダ", 70.0, true, 0.85),
    dish("日替わり定食", 90.0, true, 1.0),
];

const DINNER: [Dish; 5] = [
    dish("ポテトチップスで済ませる", 10.0, false, 0.6),
    dish("冷凍餃子と白飯", 40.0, false, 0.9),
    dish("スーパーの惣菜", 55.0, true, 0.9),
    dish("パスタとスープ", 65.0, true, 0.95),
    dish("肉野菜炒めとごはんと味噌汁", 90.0, true, 1.0),
];

pub fn menu(course: Course) -> &'static [Dish] {
    match course {
        Course::Breakfast => &BREAKFAST,
        Course::Lunch => &LUNCH,
        Course::Dinner => &DINNER,
    }
}

/// 献立を選ぶ。`effort` は 0〜1 の「まともに用意する気力」。
/// 二品くらべて、気力があれば栄養の高い方、なければ低い方を取る。
pub fn choose(course: Course, effort: f32, rng: &mut Rng) -> Dish {
    let menu = menu(course);
    let a = *rng.pick(menu);
    let b = *rng.pick(menu);
    let (low, high) = if a.nutrition <= b.nutrition {
        (a, b)
    } else {
        (b, a)
    };
    if rng.ratio() < effort.clamp(0.0, 1.0) {
        high
    } else {
        low
    }
}

/// 疲れとストレスは台所に立つ気力を削る。
/// 係数を強くしすぎると毎日カップ麺になるので、効くが支配はしない程度にする。
pub fn effort(stress: f32, sleepiness: f32) -> f32 {
    (1.0 - (0.45 * stress + 0.25 * sleepiness) / 100.0).clamp(0.1, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 食事の枠は三つで重ならない() {
        let mut seen = [0; 3];
        for h in 0..24 {
            if let Some(c) = Course::from_hour(h) {
                seen[c.index()] += 1;
            }
        }
        assert!(seen.iter().all(|&n| n > 0), "使われない枠がある");
        assert_eq!(Course::from_hour(3), None);
        assert_eq!(Course::from_hour(15), None);
        assert_eq!(Course::from_hour(22), None);
    }

    #[test]
    fn 枠の外では食事の適切さが零() {
        for h in 0..24 {
            let inside = Course::from_hour(h).is_some();
            assert_eq!(
                Course::appropriateness(h) > 0.0,
                inside,
                "{h}時の扱いが枠と食い違う"
            );
        }
    }

    #[test]
    fn 献立は栄養の低い順に並んでいる() {
        for c in Course::ALL {
            let m = menu(c);
            assert!(
                m.windows(2).all(|w| w[0].nutrition <= w[1].nutrition),
                "{} の並びが崩れている",
                c.label()
            );
        }
    }

    #[test]
    fn 気力があるほど良いものを食べる() {
        let mut rng = Rng::new(1);
        let mean = |effort: f32, rng: &mut Rng| {
            let n = 2_000;
            (0..n)
                .map(|_| choose(Course::Dinner, effort, rng).nutrition)
                .sum::<f32>()
                / n as f32
        };
        let 元気 = mean(0.95, &mut rng);
        let 疲弊 = mean(0.05, &mut rng);
        assert!(元気 > 疲弊 + 15.0, "元気 {元気} / 疲弊 {疲弊}");
    }

    #[test]
    fn 軽い献立は空腹を残す() {
        let 定食 = dish("定食", 90.0, true, 1.0);
        let 珈琲 = dish("コーヒーだけ", 5.0, false, 0.25);
        assert_eq!(定食.hunger_floor(), 0.0);
        assert!(珈琲.hunger_floor() > 70.0);
    }

    #[test]
    fn 疲れとストレスが気力を削る() {
        assert!(effort(0.0, 0.0) > effort(80.0, 80.0));
        assert!(effort(100.0, 100.0) > 0.0);
        assert!(effort(0.0, 0.0) <= 1.0);
    }
}
