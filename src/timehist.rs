use std::fmt;

use chrono::{DateTime, Duration, FixedOffset};
use yansi::Color::{Blue, Green, Red};

#[derive(Debug)]
struct TimeBucket {
    start: DateTime<FixedOffset>,
    count: usize,
}

// TODO: use trait for Bucket and TimeBucket
impl TimeBucket {
    fn new(start: DateTime<FixedOffset>) -> TimeBucket {
        TimeBucket { start, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

#[derive(Debug)]
pub struct TimeHistogram {
    vec: Vec<TimeBucket>,
    min: DateTime<FixedOffset>,
    max: DateTime<FixedOffset>,
    step: Duration,
    top: usize,
    last: usize,
    nanos: u64,
}

// TODO: use trait for Histogram and TimeHistogram
impl TimeHistogram {
    pub fn new(size: usize, ts: &[DateTime<FixedOffset>]) -> TimeHistogram {
        let mut vec = Vec::<TimeBucket>::with_capacity(size);
        let min = ts.iter().min().unwrap().clone();
        let max = ts.iter().max().unwrap().clone();
        let step = max - min;
        let inc = step / size as i32;
        for i in 0..size {
            vec.push(TimeBucket::new(min + (inc * i as i32)));
        }
        TimeHistogram {
            vec,
            min,
            max,
            step,
            top: 0,
            last: size - 1,
            nanos: (max - min).num_microseconds().unwrap() as u64,
        }
    }

    pub fn load(&mut self, vec: &[DateTime<FixedOffset>]) {
        for x in vec {
            self.add(*x);
        }
    }

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
            Some(((x * self.vec.len() as u64 / self.nanos) as usize).min(self.last))
        }
    }

    fn date_fmt_string(&self) -> &str {
        match self.step.num_seconds() {
            x if x > 86400 => "%Y-%m-%d %H:%M:%S",
            x if x > 300 => "%H:%M:%S",
            x if x > 1 => "%H:%M:%S%.3f",
            _ => "%H:%M:%S%.6f",
        }
    }
}

impl fmt::Display for TimeHistogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(100);
        let divisor = 1.max(self.top / width);
        let width_count = format!("{}", self.top).len();
        writeln!(
            f,
            "Matches: {}.",
            Blue.paint(format!(
                "{}",
                self.vec.iter().map(|r| r.count).sum::<usize>()
            )),
        )?;
        writeln!(
            f,
            "Each {} represents a count of {}",
            Red.paint("∎"),
            Blue.paint(divisor.to_string()),
        )?;
        let fmt = self.date_fmt_string();
        for row in self.vec.iter() {
            // println!("ROW");
            // println!("COUNT {}", row.count);
            // println!("WIDTH {}", row.count / divisor);
            // println!("WIDTH2 {:A<width$}", "", width = row.count / divisor);
            // println!("LABEL1 {}", row.start);
            // println!("LABEFMT {}", self.date_fmt_string());
            // println!("LABEL2 {}", row.start.format(self.date_fmt_string()));
            writeln!(
                f,
                "[{label}] [{count}] {bar}",
                label = Blue.paint(format!("{}", row.start.format(fmt))),
                count = Green.paint(format!("{:width$}", row.count, width = width_count)),
                bar = Red.paint(format!("{:∎<width$}", "", width = row.count / divisor))
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
        let mut th = TimeHistogram::new(3, &vec);
        th.load(&vec);
        println!("{}", th);
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
        let mut th = TimeHistogram::new(4, &vec);
        th.load(&vec);
        println!("{}", th);
        println!("{:#?}", th);
        let display = format!("{}", th);
        assert!(display.contains("Matches: 3"));
        assert!(display.contains("represents a count of 1"));
        assert!(display.contains("[04:25:00.001000] [2] ∎∎\n"));
        assert!(display.contains("[04:25:00.002250] [0] \n"));
        assert!(display.contains("[04:25:00.003500] [0] \n"));
        assert!(display.contains("[04:25:00.004750] [1] ∎\n"));
    }
}
