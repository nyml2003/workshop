use time::OffsetDateTime;
use workc_application::ports::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
