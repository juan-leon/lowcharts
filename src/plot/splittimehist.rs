use std::fmt;

use chrono::{DateTime, Duration, FixedOffset};
use yansi::Color::{Blue, Cyan, Green, Magenta, Red};

use crate::format::{HorizontalScale, BAR_CHAR};
use crate::plot::date_fmt_string;

const COLORS: &[yansi::Color] = &[Red, Blue, Magenta, Green, Cyan];

#[derive(Debug)]
struct TimeBucket {
    start: DateTime<FixedOffset>,
    count: Vec<usize>,
}

impl TimeBucket {
    fn new(start: DateTime<FixedOffset>, counts: usize) -> TimeBucket {
        TimeBucket {
            start,
            count: vec![0; counts],
        }
    }

    fn inc(&mut self, index: usize) {
        self.count[index] += 1;
    }

    fn total(&self) -> usize {
        self.count.iter().sum::<usize>()
    }
}

#[derive(Debug)]
/// A struct holding data to plot a split time histogram, where the display
/// shows the frequency of selected terms over time.
pub struct SplitTimeHistogram {
    vec: Vec<TimeBucket>,
    strings: Vec<String>,
    min: DateTime<FixedOffset>,
    max: DateTime<FixedOffset>,
    step: Duration,
    last: usize,
    nanos: u64,
}

impl SplitTimeHistogram {
    /// Creates a `SplitTimeHistogram` from a vector of `strings` (the terms whose
    /// frequency we want to display) and a vector of timestamps where the terms
    /// appear.
    ///
    /// `size` is the number of time slots in the histogram.  Parameter 'ts' is
    /// a slice of tuples of `DateTime` (the timestamp of a term occurrence) and
    /// the index of the term in the `strings` parameter.
    pub fn new(
        size: usize,
        strings: Vec<String>,
        ts: &[(DateTime<FixedOffset>, usize)],
    ) -> SplitTimeHistogram {
        let mut vec = Vec::<TimeBucket>::with_capacity(size);
        let min = ts.iter().min().unwrap().0;
        let max = ts.iter().max().unwrap().0;
        let step = max - min;
        let inc = step / size as i32;
        for i in 0..size {
            vec.push(TimeBucket::new(min + (inc * i as i32), strings.len()));
        }
        let mut sth = SplitTimeHistogram {
            vec,
            strings,
            min,
            max,
            step,
            last: size - 1,
            nanos: (max - min).num_microseconds().unwrap() as u64,
        };
        sth.load(ts);
        sth
    }

    /// Add to the `SplitTimeHistogram` data the values of a slice of tuples of
    /// `DateTime` (the timestamp of a term occurrence) and the index of the term
    /// in the in the list of common terms.
    pub fn load(&mut self, vec: &[(DateTime<FixedOffset>, usize)]) {
        for x in vec {
            self.add(x.0, x.1);
        }
    }

    /// Add to the `SplitTimeHistogram` data another data point (a timestamp and
    /// index of the term in the list of common terms).
    pub fn add(&mut self, ts: DateTime<FixedOffset>, index: usize) {
        if let Some(slot) = self.find_slot(ts) {
            self.vec[slot].inc(index);
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

    // Clippy gets badly confused because self.strings and COLORS may have
    // different lengths
    #[allow(clippy::needless_range_loop)]
    fn fmt_row(
        &self,
        f: &mut fmt::Formatter,
        row: &TimeBucket,
        divisor: usize,
        widths: &[usize],
        ts_fmt: &str,
    ) -> fmt::Result {
        write!(
            f,
            "[{}] [",
            Blue.paint(format!("{}", row.start.format(ts_fmt)))
        )?;
        for i in 0..self.strings.len() {
            write!(
                f,
                "{}",
                COLORS[i].paint(format!("{:width$}", row.count[i], width = widths[i]))
            )?;
            if i < self.strings.len() - 1 {
                write!(f, "/")?;
            }
        }
        write!(f, "] ")?;
        for i in 0..self.strings.len() {
            write!(
                f,
                "{}",
                COLORS[i].paint(BAR_CHAR.repeat(row.count[i] / divisor).to_string())
            )?;
        }
        writeln!(f)
    }
}

impl fmt::Display for SplitTimeHistogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(100);
        let total = self.vec.iter().map(|r| r.total()).sum::<usize>();
        let top = self.vec.iter().map(|r| r.total()).max().unwrap_or(1);
        let horizontal_scale = HorizontalScale::new(top / width);
        // These are the widths of every count column
        let widths: Vec<usize> = (0..self.strings.len())
            .map(|i| {
                self.vec
                    .iter()
                    .map(|r| r.count[i].to_string().len())
                    .max()
                    .unwrap()
            })
            .collect();

        writeln!(f, "Matches: {}.", total)?;
        for (i, s) in self.strings.iter().enumerate() {
            let total = self.vec.iter().map(|r| r.count[i]).sum::<usize>();
            writeln!(f, "{}: {}.", COLORS[i].paint(s), total)?;
        }
        writeln!(f, "{}", horizontal_scale)?;
        let ts_fmt = date_fmt_string(self.step.num_seconds());
        for row in self.vec.iter() {
            self.fmt_row(f, row, horizontal_scale.get_scale(), &widths, ts_fmt)?;
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
        let mut vec = Vec::<(DateTime<FixedOffset>, usize)>::new();
        vec.push((
            DateTime::parse_from_rfc3339("2021-04-15T04:25:00+00:00").unwrap(),
            1,
        ));
        vec.push((
            DateTime::parse_from_rfc3339("2022-04-15T04:25:00+00:00").unwrap(),
            1,
        ));
        vec.push((
            DateTime::parse_from_rfc3339("2022-04-15T04:25:00+00:00").unwrap(),
            0,
        ));
        vec.push((
            DateTime::parse_from_rfc3339("2022-04-15T04:25:00+00:00").unwrap(),
            2,
        ));
        for _ in 0..11 {
            vec.push((
                DateTime::parse_from_rfc3339("2023-04-15T04:25:00+00:00").unwrap(),
                2,
            ));
        }
        let th = SplitTimeHistogram::new(
            3,
            vec!["one".to_string(), "two".to_string(), "three".to_string()],
            &vec,
        );
        println!("{}", th);
        let display = format!("{}", th);
        assert!(display.contains("Matches: 15"));
        assert!(display.contains("one: 1."));
        assert!(display.contains("two: 2."));
        assert!(display.contains("three: 12."));
        assert!(display.contains("represents a count of 1"));
        assert!(display.contains("[2021-04-15 04:25:00] [0/1/ 0] ∎\n"));
        assert!(display.contains("[2021-12-14 12:25:00] [1/1/ 1] ∎∎∎\n"));
        assert!(display.contains("[2022-08-14 20:25:00] [0/0/11] ∎∎∎∎∎∎∎∎∎∎∎\n"));
    }
}
