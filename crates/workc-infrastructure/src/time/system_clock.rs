use time::OffsetDateTime;
use workc_application::ports::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_returns_current_time() {
        let clock = SystemClock;
        let before = OffsetDateTime::now_utc();
        let now = clock.now();
        let after = OffsetDateTime::now_utc();
        assert!(now >= before);
        assert!(now <= after);
    }
}
