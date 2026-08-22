//! 恒常性の欲求。すべて 0.0(満たされている)〜100.0(限界)で表す。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drive {
    /// 眠気
    Sleepiness,
    /// 空腹
    Hunger,
    /// 寂しさ
    Loneliness,
    /// 不潔感
    Grime,
    /// ストレス
    Stress,
}

impl Drive {
    pub const ALL: [Drive; 5] = [
        Drive::Sleepiness,
        Drive::Hunger,
        Drive::Loneliness,
        Drive::Grime,
        Drive::Stress,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Drive::Sleepiness => "眠気",
            Drive::Hunger => "空腹",
            Drive::Loneliness => "寂しさ",
            Drive::Grime => "不潔感",
            Drive::Stress => "ストレス",
        }
    }

    /// 覚醒中の 1 分あたりの基礎上昇量。
    fn base_drift(self) -> f32 {
        match self {
            // 16 時間で眠気がほぼ限界に達する
            Drive::Sleepiness => 100.0 / (16.0 * 60.0),
            // 5 時間で空腹が限界に達する
            Drive::Hunger => 100.0 / (5.0 * 60.0),
            Drive::Loneliness => 100.0 / (30.0 * 60.0),
            Drive::Grime => 100.0 / (20.0 * 60.0),
            // ストレスは放置しても緩やかに溜まる
            Drive::Stress => 100.0 / (48.0 * 60.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Needs {
    pub sleepiness: f32,
    pub hunger: f32,
    pub loneliness: f32,
    pub grime: f32,
    pub stress: f32,
}

impl Needs {
    /// 朝、十分に眠って起きた直後の状態。
    pub fn rested() -> Self {
        Self {
            sleepiness: 5.0,
            hunger: 45.0,
            loneliness: 20.0,
            grime: 15.0,
            stress: 10.0,
        }
    }

    pub fn get(&self, d: Drive) -> f32 {
        match d {
            Drive::Sleepiness => self.sleepiness,
            Drive::Hunger => self.hunger,
            Drive::Loneliness => self.loneliness,
            Drive::Grime => self.grime,
            Drive::Stress => self.stress,
        }
    }

    pub fn set(&mut self, d: Drive, v: f32) {
        let v = v.clamp(0.0, 100.0);
        match d {
            Drive::Sleepiness => self.sleepiness = v,
            Drive::Hunger => self.hunger = v,
            Drive::Loneliness => self.loneliness = v,
            Drive::Grime => self.grime = v,
            Drive::Stress => self.stress = v,
        }
    }

    pub fn adjust(&mut self, d: Drive, delta: f32) {
        self.set(d, self.get(d) + delta);
    }

    /// 何もしない 1 分で溜まっていく分。`circadian` は概日リズムによる
    /// 眠気の増減係数(夜ほど大きい)。
    pub fn drift(&mut self, minutes: u32, circadian: f32) {
        let m = minutes as f32;
        for d in Drive::ALL {
            let mult = if d == Drive::Sleepiness {
                circadian
            } else {
                1.0
            };
            self.adjust(d, d.base_drift() * m * mult);
        }
    }

    /// 最も逼迫している欲求とその値。
    pub fn most_urgent(&self) -> (Drive, f32) {
        let mut worst = (Drive::ALL[0], self.get(Drive::ALL[0]));
        for d in Drive::ALL {
            let v = self.get(d);
            if v > worst.1 {
                worst = (d, v);
            }
        }
        worst
    }

    /// 気分。100 が絶好調、0 が限界。
    pub fn mood(&self) -> f32 {
        let burden = 0.30 * self.sleepiness
            + 0.25 * self.hunger
            + 0.15 * self.loneliness
            + 0.10 * self.grime
            + 0.20 * self.stress;
        (100.0 - burden).clamp(0.0, 100.0)
    }

    pub fn mood_label(&self) -> &'static str {
        match self.mood() {
            m if m >= 80.0 => "上機嫌",
            m if m >= 65.0 => "ふつう",
            m if m >= 45.0 => "少し疲れ気味",
            m if m >= 25.0 => "しんどい",
            _ => "限界",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 欲求は範囲外に出ない() {
        let mut n = Needs::rested();
        n.adjust(Drive::Hunger, 1_000.0);
        assert_eq!(n.hunger, 100.0);
        n.adjust(Drive::Hunger, -1_000.0);
        assert_eq!(n.hunger, 0.0);
    }

    #[test]
    fn 放置すると欲求が溜まる() {
        let mut n = Needs::rested();
        let before = n.hunger;
        n.drift(120, 1.0);
        assert!(n.hunger > before);
    }

    #[test]
    fn 十六時間起きていると眠気が限界に達する() {
        let mut n = Needs::rested();
        n.drift(16 * 60, 1.0);
        assert!(n.sleepiness > 95.0, "眠気 {}", n.sleepiness);
    }

    #[test]
    fn 概日リズムが眠気の速度を変える() {
        let mut 昼 = Needs::rested();
        let mut 夜 = Needs::rested();
        昼.drift(60, 0.5);
        夜.drift(60, 2.0);
        assert!(夜.sleepiness > 昼.sleepiness);
    }

    #[test]
    fn 最も逼迫した欲求を選ぶ() {
        let mut n = Needs::rested();
        n.set(Drive::Loneliness, 99.0);
        assert_eq!(n.most_urgent().0, Drive::Loneliness);
    }

    #[test]
    fn 欲求が満たされているほど気分が良い() {
        let 良 = Needs::rested();
        let mut 悪 = Needs::rested();
        for d in Drive::ALL {
            悪.set(d, 100.0);
        }
        assert!(良.mood() > 悪.mood());
        assert_eq!(悪.mood(), 0.0);
    }
}
