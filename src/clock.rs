//! シミュレーション内の時計。分解能は 1 分、起点は月曜 00:00。

pub const MINUTES_PER_DAY: u32 = 24 * 60;
pub const DAYS_PER_WEEK: u32 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    pub fn from_index(i: u32) -> Self {
        match i % DAYS_PER_WEEK {
            0 => Weekday::Mon,
            1 => Weekday::Tue,
            2 => Weekday::Wed,
            3 => Weekday::Thu,
            4 => Weekday::Fri,
            5 => Weekday::Sat,
            _ => Weekday::Sun,
        }
    }

    pub fn is_weekend(self) -> bool {
        matches!(self, Weekday::Sat | Weekday::Sun)
    }

    pub fn label(self) -> &'static str {
        match self {
            Weekday::Mon => "月",
            Weekday::Tue => "火",
            Weekday::Wed => "水",
            Weekday::Thu => "木",
            Weekday::Fri => "金",
            Weekday::Sat => "土",
            Weekday::Sun => "日",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    /// 起点からの経過分。
    pub elapsed: u32,
}

impl Clock {
    pub fn start() -> Self {
        Self { elapsed: 0 }
    }

    pub fn advance(&mut self, minutes: u32) {
        self.elapsed += minutes;
    }

    /// 0 始まりの通算日。
    pub fn day(self) -> u32 {
        self.elapsed / MINUTES_PER_DAY
    }

    pub fn minute_of_day(self) -> u32 {
        self.elapsed % MINUTES_PER_DAY
    }

    pub fn hour(self) -> u32 {
        self.minute_of_day() / 60
    }

    pub fn minute(self) -> u32 {
        self.minute_of_day() % 60
    }

    pub fn weekday(self) -> Weekday {
        Weekday::from_index(self.day())
    }

    /// 「06:30」形式。
    pub fn hhmm(self) -> String {
        format!("{:02}:{:02}", self.hour(), self.minute())
    }

    /// 「3日目(木) 06:30」形式。
    pub fn stamp(self) -> String {
        format!(
            "{}日目({}) {}",
            self.day() + 1,
            self.weekday().label(),
            self.hhmm()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 起点は月曜零時() {
        let c = Clock::start();
        assert_eq!(c.hour(), 0);
        assert_eq!(c.minute(), 0);
        assert_eq!(c.weekday(), Weekday::Mon);
        assert_eq!(c.day(), 0);
    }

    #[test]
    fn 日付が繰り上がる() {
        let mut c = Clock::start();
        c.advance(MINUTES_PER_DAY + 90);
        assert_eq!(c.day(), 1);
        assert_eq!(c.hhmm(), "01:30");
        assert_eq!(c.weekday(), Weekday::Tue);
    }

    #[test]
    fn 週末を判定する() {
        let mut c = Clock::start();
        c.advance(MINUTES_PER_DAY * 5); // 土曜
        assert!(c.weekday().is_weekend());
        c.advance(MINUTES_PER_DAY * 2); // 翌週月曜
        assert!(!c.weekday().is_weekend());
        assert_eq!(c.weekday(), Weekday::Mon);
    }

    #[test]
    fn stamp_は一日目から数える() {
        assert_eq!(Clock::start().stamp(), "1日目(月) 00:00");
    }
}
