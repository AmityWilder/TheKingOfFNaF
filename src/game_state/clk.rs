use crate::{DECISECS_PER_HOUR, DECISECS_PER_SEC, SECS_PER_HOUR, SECS_PER_MIN};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockTime {
    /// One hour is 45 seconds. A night is 4 minutes 30 seconds, or 270 seconds -- 2700 deciseconds.
    /// This can be expressed in 12 bits as 0b101010001100.
    pub deciseconds: u16,
    pub pings_since_change: u32,
}

impl PartialOrd for ClockTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClockTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deciseconds.cmp(&other.deciseconds)
    }
}

impl Default for ClockTime {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ClockTime {
    pub const MAX: u16 = 2700;

    pub const fn new(deciseconds: u16) -> Self {
        Self {
            deciseconds,
            pings_since_change: 0,
        }
    }

    pub const fn deciseconds(&self) -> u16 {
        self.deciseconds
    }

    pub const fn pings_since_change(&self) -> u32 {
        self.pings_since_change
    }

    /// It takes 1 bit more than a char to describe the number of seconds in a night.
    pub const fn seconds(&self) -> u16 {
        self.deciseconds / DECISECS_PER_SEC as u16
    }

    /// Not sure what we'd need this for, but just in case.
    pub const fn minutes(&self) -> u16 {
        self.seconds() / SECS_PER_MIN as u16 // realtime
    }

    /// What hour of the night we are at
    pub const fn hour(&self) -> u16 {
        self.seconds() / SECS_PER_HOUR as u16 // gametime
    }

    /// Converts hours to deciseconds, for finding how many deciseconds we are through the current hour.
    pub const fn whole_hour_deciseconds(&self) -> u16 {
        self.hour() * DECISECS_PER_HOUR
    }

    /// Finds how many deciseconds into the current hour we are.
    pub const fn deciseconds_since_hour(&self) -> u16 {
        self.deciseconds() - self.whole_hour_deciseconds()
    }

    /// Updates [`Self::pings_since_change`] if the time is a duplicate or erroneous
    pub const fn update_time(&mut self, new_time: u16) {
        if new_time > self.deciseconds
            && new_time < 6000
            && new_time > 0
            && ((new_time - self.deciseconds) < 10 || self.pings_since_change > 10)
        {
            self.deciseconds = new_time;
            self.pings_since_change = 0;
        } else {
            self.pings_since_change += 1;
        }
    }
}

impl std::fmt::Display for ClockTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:02}.{}",
            self.minutes(),
            self.seconds() % SECS_PER_MIN as u16,
            self.deciseconds() % DECISECS_PER_SEC as u16
        )
    }
}
