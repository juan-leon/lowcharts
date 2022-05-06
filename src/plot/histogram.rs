use std::fmt;
use std::ops::Range;

use yansi::Color::{Blue, Green, Red};

use crate::format::F64Formatter;
use crate::stats::Stats;

#[derive(Debug)]
/// A struct that represents a bucket of an histogram.
struct Bucket {
    range: Range<f64>,
    count: usize,
}

impl Bucket {
    fn new(range: Range<f64>) -> Bucket {
        Bucket { range, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

#[derive(Debug)]
/// A struct holding data to plot a Histogram of numerical data.
pub struct Histogram {
    vec: Vec<Bucket>,
    max: f64,
    step: f64,
    top: usize,
    last: usize,
    stats: Stats,
    precision: Option<usize>, // If None, then human friendly display will be used
}

impl Histogram {
    /// Creates a Histogram from a vector of numerical data.
    ///
    /// `intervals` is the number of histogram buckets to display (capped to the
    /// length of input data).
    ///
    /// `precision` is an Option with the number of decimals to display.  If
    /// "None" is used, human units will be used, with an heuristic based on the
    /// input data for deciding the units and the decimal places.
    pub fn new(vec: &[f64], intervals: usize, precision: Option<usize>) -> Histogram {
        let stats = Stats::new(vec, precision);
        let size = intervals.min(vec.len());
        let step = (stats.max - stats.min) / size as f64;
        let mut histogram = Histogram::new_with_stats(size, step, stats, precision);
        histogram.load(vec);
        histogram
    }

    /// Creates a Histogram with no input data.
    ///
    ///
    /// Parameters are similar to those on the `new` method, but a parameter
    /// named `stats` is needed to decide how future data (to be injected with
    /// the load method) will be accommodated.
    pub fn new_with_stats(
        size: usize,
        step: f64,
        stats: Stats,
        precision: Option<usize>,
    ) -> Histogram {
        let mut vec = Vec::<Bucket>::with_capacity(size);
        let mut lower = stats.min;
        for _ in 0..size {
            vec.push(Bucket::new(lower..lower + step));
            lower += step;
        }
        Histogram {
            vec,
            max: stats.min + (step * size as f64),
            step,
            top: 0,
            last: size - 1,
            stats,
            precision,
        }
    }

    /// Add to the `Histogram` data the values of a slice of numerical data.
    pub fn load(&mut self, vec: &[f64]) {
        for x in vec {
            self.add(*x);
        }
    }

    /// Add to the `Histogram` a single piece of numerical data.
    pub fn add(&mut self, n: f64) {
        if let Some(slot) = self.find_slot(n) {
            self.vec[slot].inc();
            self.top = self.top.max(self.vec[slot].count);
        }
    }

    fn find_slot(&self, n: f64) -> Option<usize> {
        if n < self.stats.min || n > self.max {
            None
        } else {
            Some((((n - self.stats.min) / self.step) as usize).min(self.last))
        }
    }
}

impl fmt::Display for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stats)?;
        let formatter = match self.precision {
            None => F64Formatter::new_with_range(self.stats.min..self.stats.max),
            Some(n) => F64Formatter::new(n),
        };
        let writer = HistWriter {
            width: f.width().unwrap_or(110),
            formatter,
        };
        writer.write(f, self)
    }
}

struct HistWriter {
    width: usize,
    formatter: F64Formatter,
}

impl HistWriter {
    pub fn write(&self, f: &mut fmt::Formatter, hist: &Histogram) -> fmt::Result {
        let width_range = self.get_width(hist);
        let width_count = ((hist.top as f64).log10().ceil() as usize).max(1);
        let divisor = 1.max(hist.top / self.get_max_bar_len(width_range + width_count));
        writeln!(
            f,
            "each {} represents a count of {}",
            Red.paint("∎"),
            Blue.paint(divisor.to_string()),
        )?;
        for x in hist.vec.iter() {
            self.write_bucket(f, x, divisor, width_range, width_count)?;
        }
        Ok(())
    }

    fn write_bucket(
        &self,
        f: &mut fmt::Formatter,
        bucket: &Bucket,
        divisor: usize,
        width: usize,
        width_count: usize,
    ) -> fmt::Result {
        writeln!(
            f,
            "[{range}] [{count}] {bar}",
            range = Blue.paint(format!(
                "{:>width$} .. {:>width$}",
                self.formatter.format(bucket.range.start),
                self.formatter.format(bucket.range.end),
                width = width,
            )),
            count = Green.paint(format!("{:width$}", bucket.count, width = width_count)),
            bar = Red.paint(format!("{:∎<width$}", "", width = bucket.count / divisor)),
        )
    }

    fn get_width(&self, hist: &Histogram) -> usize {
        self.formatter
            .format(hist.stats.min)
            .len()
            .max(self.formatter.format(hist.max).len())
    }

    fn get_max_bar_len(&self, fixed_width: usize) -> usize {
        const EXTRA_CHARS: usize = 10;
        if self.width < fixed_width + EXTRA_CHARS {
            75
        } else {
            self.width - fixed_width - EXTRA_CHARS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yansi::Paint;

    #[test]
    fn test_buckets() {
        let stats = Stats::new(&[-2.0, 14.0], None);
        let mut hist = Histogram::new_with_stats(8, 2.5, stats, None);
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);

        assert_eq!(hist.top, 8);
        let bucket = &hist.vec[0];
        assert_eq!(bucket.range, -2.0..0.5);
        assert_eq!(bucket.count, 3);
        let bucket = &hist.vec[1];
        assert_eq!(bucket.count, 8);
        assert_eq!(bucket.range, 0.5..3.0);
    }

    #[test]
    fn test_buckets_bad_stats() {
        let mut hist = Histogram::new_with_stats(6, 1.0, Stats::new(&[-2.0, 4.0], None), None);
        hist.load(&[-1.0, 2.0, -1.0, 2.0, 10.0, 10.0, 10.0, -10.0]);
        assert_eq!(hist.top, 2);
    }

    #[test]
    fn display_test() {
        let stats = Stats::new(&[-2.0, 14.0], None);
        let mut hist = Histogram::new_with_stats(8, 2.5, stats, Some(3));
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);
        Paint::disable();
        let display = format!("{}", hist);
        assert!(display.contains("[-2.000 ..  0.500] [3] ∎∎∎\n"));
        assert!(display.contains("[ 0.500 ..  3.000] [8] ∎∎∎∎∎∎∎∎\n"));
        assert!(display.contains("[10.500 .. 13.000] [2] ∎∎\n"));
    }

    #[test]
    fn display_test_bad_width() {
        let mut hist = Histogram::new_with_stats(8, 2.5, Stats::new(&[-2.0, 14.0], None), Some(3));
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);
        Paint::disable();
        let display = format!("{:2}", hist);
        assert!(display.contains("[-2.000 ..  0.500] [3] ∎∎∎\n"));
    }

    #[test]
    fn display_test_human_units() {
        let vector = &[
            -1.0,
            -12000000.0,
            -12000001.0,
            -12000002.0,
            -12000003.0,
            -2000000.0,
            500000.0,
            500000.0,
        ];
        let hist = Histogram::new(vector, vector.len(), None);
        Paint::disable();
        let display = format!("{}", hist);
        assert!(display.contains("[-12.0 M .. -10.4 M] [4] ∎∎∎∎\n"));
        assert!(display.contains("[ -2.6 M ..  -1.1 M] [1] ∎\n"));
        assert!(display.contains("[ -1.1 M ..   0.5 M] [3] ∎∎∎\n"));
        assert!(display.contains("Samples = 8; Min = -12.0 M; Max = 0.5 M"));
        assert!(display.contains("Average = -6.1 M;"));
    }
}
