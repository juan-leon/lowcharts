use std::fmt;

use chrono::{DateTime, Duration, FixedOffset};
use yansi::Color::Blue;

use crate::format::HorizontalScale;
use crate::plot::date_fmt_string;

#[derive(Debug)]
struct TimeBucket {
    start: DateTime<FixedOffset>,
    count: usize,
}

impl TimeBucket {
    fn new(start: DateTime<FixedOffset>) -> Self {
        Self { start, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

#[derive(Debug)]
/// A struct holding data to plot a `TimeHistogram` of timestamp data.
pub struct TimeHistogram {
    vec: Vec<TimeBucket>,
    min: DateTime<FixedOffset>,
    max: DateTime<FixedOffset>,
    step: Duration,
    top: usize,
    last: usize,
    nanos: u64,
}

impl TimeHistogram {
    /// Creates a Histogram from a vector of `DateTime` elements.
    ///
    /// `size` is the number of histogram buckets to display.
    pub fn new(size: usize, ts: &[DateTime<FixedOffset>]) -> Self {
        let mut vec = Vec::<TimeBucket>::with_capacity(size);
        let min = *ts.iter().min().unwrap();
        let max = *ts.iter().max().unwrap();
        let step = max - min;
        let inc = step / size as i32;
        for i in 0..size {
            vec.push(TimeBucket::new(min + (inc * i as i32)));
        }
        let mut timehist = Self {
            vec,
            min,
            max,
            step,
            top: 0,
            last: size - 1,
            nanos: (max - min).num_microseconds().unwrap() as u64,
        };
        timehist.load(ts);
        timehist
    }

    /// Add to the `TimeHistogram` data the values of a slice of `DateTime`
    /// elements.  Elements not in the initial range (the one passed to `new`)
    /// will be silently discarded.
    pub fn load(&mut self, vec: &[DateTime<FixedOffset>]) {
        for x in vec {
            self.add(*x);
        }
    }

    /// Add to the `TimeHistogram` another `DateTime` element.  If element is not
    /// in the initial range (the one passed to `new`), it will be silently
    /// discarded.
    pub fn add(&mut self, ts: DateTime<FixedOffset>) {
        if let Some(slot) = self.find_slot(ts) {
            self.vec[slot].inc();
            self.top = self.top.max(self.vec[slot].count);
        }
    }

    fn find_slot(&self, ts: DateTime<FixedOffset>) -> Option<usize> {
        if ts < self.min || ts > self.max {
            None
        } else {
            let x = (ts - self.min).num_microseconds().unwrap() as u64;
            if self.nanos == 0 {
                // All timestamps are the same.  We will have a degenrate plot
                // (as opposed to failing hard).
                Some(0)
            } else {
                Some(((x * self.vec.len() as u64 / self.nanos) as usize).min(self.last))
            }
        }
    }
}

impl fmt::Display for TimeHistogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(100);
        let horizontal_scale = HorizontalScale::new(self.top / width);
        let width_count = format!("{}", self.top).len();
        writeln!(
            f,
            "Matches: {}.",
            Blue.paint(format!(
                "{}",
                self.vec.iter().map(|r| r.count).sum::<usize>()
            )),
        )?;
        writeln!(f, "{}", horizontal_scale)?;
        let ts_fmt = date_fmt_string(self.step.num_seconds());
        for row in self.vec.iter() {
            writeln!(
                f,
                "[{label}] [{count}] {bar}",
                label = Blue.paint(format!("{}", row.start.format(ts_fmt))),
                count = horizontal_scale.get_count(row.count, width_count),
                bar = horizontal_scale.get_bar(row.count)
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yansi::Paint;

    #[test]
    fn test_big_time_interval() {
        Paint::disable();
        let mut vec = Vec::<DateTime<FixedOffset>>::new();
        vec.push(DateTime::parse_from_rfc3339("2021-04-15T04:25:00+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2023-04-15T04:25:00+00:00").unwrap());
        let th = TimeHistogram::new(3, &vec);
        let display = format!("{}", th);
        assert!(display.contains("Matches: 5"));
        assert!(display.contains("represents a count of 1"));
        assert!(display.contains("[2021-04-15 04:25:00] [1] ∎\n"));
        assert!(display.contains("[2021-12-14 12:25:00] [3] ∎∎∎\n"));
        assert!(display.contains("[2022-08-14 20:25:00] [1] ∎\n"));
    }

    #[test]
    fn test_small_time_interval() {
        Paint::disable();
        let mut vec = Vec::<DateTime<FixedOffset>>::new();
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00.001+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00.002+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00.006+00:00").unwrap());
        let th = TimeHistogram::new(4, &vec);
        let display = format!("{}", th);
        assert!(display.contains("Matches: 3"));
        assert!(display.contains("represents a count of 1"));
        assert!(display.contains("[04:25:00.001000] [2] ∎∎\n"));
        assert!(display.contains("[04:25:00.002250] [0] \n"));
        assert!(display.contains("[04:25:00.003500] [0] \n"));
        assert!(display.contains("[04:25:00.004750] [1] ∎\n"));
    }

    #[test]
    fn test_single_timestamp() {
        Paint::disable();
        let mut vec = Vec::<DateTime<FixedOffset>>::new();
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00.001+00:00").unwrap());
        vec.push(DateTime::parse_from_rfc3339("2022-04-15T04:25:00.001+00:00").unwrap());
        let th = TimeHistogram::new(4, &vec);
        let display = format!("{}", th);
        assert!(display.contains("Matches: 2"));
        assert!(display.contains("represents a count of 1"));
        assert!(display.contains("[04:25:00.001000] [2] ∎∎\n"));
        assert!(display.contains("[04:25:00.001000] [0] \n"));
    }
}
