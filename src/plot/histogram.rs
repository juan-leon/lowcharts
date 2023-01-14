use std::fmt;
use std::ops::Range;

use yansi::Color::Blue;

use crate::format::{F64Formatter, HorizontalScale};
use crate::stats::Stats;

#[derive(Debug)]
/// A struct that represents a bucket of an histogram.
struct Bucket {
    range: Range<f64>,
    count: usize,
}

impl Bucket {
    fn new(range: Range<f64>) -> Self {
        Self { range, count: 0 }
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

/// A struct representing the options to build an histogram.
pub struct Histogram {
    vec: Vec<Bucket>,
    step: f64,
    // Maximum of all bucket counts
    top: usize,
    last: usize,
    stats: Stats,
    log_scale: bool,
    precision: Option<usize>, // If None, then human friendly display will be used
}

/// A struct holding data to plot a Histogram of numerical data.
#[derive(Default)]
pub struct HistogramOptions {
    /// `intervals` is the number of histogram buckets to display (capped to the
    /// length of input data).
    pub intervals: usize,
    /// If true, logarithmic scale will be used for buckets
    pub log_scale: bool,
    /// `precision` is an Option with the number of decimals to display.  If
    /// "None" is used, human units will be used, with an heuristic based on the
    /// input data for deciding the units and the decimal places.
    pub precision: Option<usize>,
}

impl Histogram {
    /// Creates a Histogram from a vector of numerical data.
    ///
    /// `options` is a `HistogramOptions` struct with the preferences to create
    /// histogram.
    pub fn new(vec: &[f64], mut options: HistogramOptions) -> Self {
        let mut stats = Stats::new(vec, options.precision);
        if options.log_scale {
            stats.min = 0.0; // We will silently discard negative values
        }
        options.intervals = options.intervals.clamp(1, vec.len());
        let mut histogram = Self::new_with_stats(stats, &options);
        histogram.load(vec);
        histogram
    }

    /// Creates a Histogram with no input data.
    ///
    /// Parameters are similar to those on the `new` method, but a parameter
    /// named `stats` is needed to decide how future data (to be injected with
    /// the load method) will be accommodated.
    pub fn new_with_stats(stats: Stats, options: &HistogramOptions) -> Self {
        let step = if options.log_scale {
            f64::NAN
        } else {
            (stats.max - stats.min) / options.intervals as f64
        };
        Self {
            vec: Self::build_buckets(stats.min..stats.max, options),
            step,
            top: 0,
            last: options.intervals - 1,
            stats,
            log_scale: options.log_scale,
            precision: options.precision,
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
        if n < self.stats.min || n > self.stats.max {
            return None;
        }
        if self.log_scale {
            let mut bucket = None;
            for i in 0..self.vec.len() {
                if self.vec[i].range.end >= n {
                    bucket = Some(i);
                    break;
                }
            }
            bucket
        } else {
            Some((((n - self.stats.min) / self.step) as usize).min(self.last))
        }
    }

    fn build_buckets(range: Range<f64>, options: &HistogramOptions) -> Vec<Bucket> {
        let mut vec = Vec::<Bucket>::with_capacity(options.intervals);
        if options.log_scale {
            let first_bucket_size = range.end / (2_f64.powi(options.intervals as i32) - 1.0);
            let mut lower = 0.0;
            for i in 0..options.intervals {
                let upper = lower + 2_f64.powi(i as i32) * first_bucket_size;
                vec.push(Bucket::new(lower..upper));
                lower = upper;
            }
        } else {
            let step = (range.end - range.start) / options.intervals as f64;
            let mut lower = range.start;
            for _ in 0..options.intervals {
                vec.push(Bucket::new(lower..lower + step));
                lower += step;
            }
        }
        vec
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
        let horizontal_scale =
            HorizontalScale::new(hist.top / self.get_max_bar_len(width_range + width_count));
        writeln!(f, "{horizontal_scale}")?;
        for x in &hist.vec {
            self.write_bucket(f, x, &horizontal_scale, width_range, width_count)?;
        }
        Ok(())
    }

    fn write_bucket(
        &self,
        f: &mut fmt::Formatter,
        bucket: &Bucket,
        horizontal_scale: &HorizontalScale,
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
            count = horizontal_scale.get_count(bucket.count, width_count),
            bar = horizontal_scale.get_bar(bucket.count)
        )
    }

    fn get_width(&self, hist: &Histogram) -> usize {
        self.formatter
            .format(hist.stats.min)
            .len()
            .max(self.formatter.format(hist.stats.max).len())
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
    use float_eq::assert_float_eq;
    use yansi::Paint;

    #[test]
    fn test_buckets() {
        let stats = Stats::new(&[-2.0, 14.0], None);
        let options = HistogramOptions {
            intervals: 8,
            ..Default::default()
        };
        let mut hist = Histogram::new_with_stats(stats, &options);
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);

        assert_eq!(hist.top, 5);
        let bucket = &hist.vec[0];
        assert_eq!(bucket.range, -2.0..0.0);
        assert_eq!(bucket.count, 3);
        let bucket = &hist.vec[1];
        assert_eq!(bucket.count, 5);
        assert_eq!(bucket.range, 0.0..2.0);
    }

    #[test]
    fn test_buckets_bad_stats() {
        let options = HistogramOptions {
            intervals: 6,
            ..Default::default()
        };
        let mut hist = Histogram::new_with_stats(Stats::new(&[-2.0, 4.0], None), &options);
        hist.load(&[-1.0, 2.0, -1.0, 2.0, 10.0, 10.0, 10.0, -10.0]);
        assert_eq!(hist.top, 2);
    }

    #[test]
    fn display_test() {
        let stats = Stats::new(&[-2.0, 14.0], None);
        let options = HistogramOptions {
            intervals: 8,
            precision: Some(3),
            ..Default::default()
        };
        let mut hist = Histogram::new_with_stats(stats, &options);
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);
        Paint::disable();
        let display = format!("{hist}");
        assert!(display.contains("[-2.000 ..  0.000] [3] ∎∎∎\n"));
        assert!(display.contains("[ 0.000 ..  2.000] [5] ∎∎∎∎∎\n"));
        assert!(display.contains("[ 2.000 ..  4.000] [3] ∎∎∎\n"));
        assert!(display.contains("[ 6.000 ..  8.000] [0] \n"));
        assert!(display.contains("[10.000 .. 12.000] [2] ∎∎\n"));
    }

    #[test]
    fn display_test_bad_width() {
        let options = HistogramOptions {
            intervals: 8,
            precision: Some(3),
            ..Default::default()
        };
        let mut hist = Histogram::new_with_stats(Stats::new(&[-2.0, 14.0], None), &options);
        hist.load(&[
            -1.0, -1.1, 2.0, 2.0, 2.1, -0.9, 11.0, 11.2, 1.9, 1.99, 1.98, 1.97, 1.96,
        ]);
        Paint::disable();
        let display = format!("{hist:2}");
        assert!(display.contains("[-2.000 ..  0.000] [3] ∎∎∎\n"));
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
        let hist = Histogram::new(
            vector,
            HistogramOptions {
                intervals: 10,
                ..Default::default()
            },
        );
        Paint::disable();
        let display = format!("{hist}");
        assert!(display.contains("[-12.0 M .. -10.4 M] [4] ∎∎∎∎\n"));
        assert!(display.contains("[ -2.6 M ..  -1.1 M] [1] ∎\n"));
        assert!(display.contains("[ -1.1 M ..   0.5 M] [3] ∎∎∎\n"));
        assert!(display.contains("Samples = 8; Min = -12.0 M; Max = 0.5 M"));
        assert!(display.contains("Average = -6.1 M;"));
    }

    #[test]
    fn display_test_log_scale() {
        let hist = Histogram::new(
            &[0.4, 0.4, 0.4, 0.4, 255.0, 0.2, 1.2, 128.0, 126.0, -7.0],
            HistogramOptions {
                intervals: 8,
                log_scale: true,
                ..Default::default()
            },
        );
        Paint::disable();
        let display = format!("{hist}");
        assert!(display.contains("[  0.00 ..   1.00] [5] ∎∎∎∎∎\n"));
        assert!(display.contains("[  1.00 ..   3.00] [1] ∎\n"));
        assert!(display.contains("[  3.00 ..   7.00] [0]"));
        assert!(display.contains("[  7.00 ..  15.00] [0]"));
        assert!(display.contains("[ 15.00 ..  31.00] [0]"));
        assert!(display.contains("[ 31.00 ..  63.00] [0]"));
        assert!(display.contains("[ 63.00 .. 127.00] [1] ∎\n"));
        assert!(display.contains("[127.00 .. 255.00] [2] ∎∎\n"));
    }

    #[test]
    fn build_buckets_log_scale() {
        let options = HistogramOptions {
            intervals: 8,
            log_scale: true,
            ..Default::default()
        };
        let buckets = Histogram::build_buckets(0.0..2.0_f64.powi(8) - 1.0, &options);
        assert!(buckets.len() == 8);
        assert!(buckets[0].range == (0.0..1.0));
        assert!(buckets[1].range == (1.0..3.0));
        assert!(buckets[2].range == (3.0..7.0));
        assert!(buckets[3].range == (7.0..15.0));
        assert!(buckets[4].range == (15.0..31.0));
        assert!(buckets[5].range == (31.0..63.0));
        assert!(buckets[6].range == (63.0..127.0));
        assert!(buckets[7].range == (127.0..255.0));
    }

    #[test]
    fn build_buckets_log_scale_with_math() {
        let options = HistogramOptions {
            intervals: 10,
            log_scale: true,
            ..Default::default()
        };
        let buckets = Histogram::build_buckets(0.0..10000.0, &options);
        assert!(buckets.len() == 10);
        for i in 0..9 {
            assert_float_eq!(
                2.0 * (buckets[i].range.end - buckets[i].range.start),
                buckets[i + 1].range.end - buckets[i + 1].range.start,
                rmax <= 2.0 * f64::EPSILON
            );
        }
        assert_float_eq!(
            buckets[9].range.end - buckets[0].range.start,
            10000.0,
            rmax <= 2.0 * f64::EPSILON
        );
    }

    #[test]
    fn build_buckets_no_log_scale() {
        let options = HistogramOptions {
            intervals: 7,
            ..Default::default()
        };
        let buckets = Histogram::build_buckets(0.0..700.0, &options);
        assert!(buckets.len() == 7);
        for i in 0..6 {
            let min = (i * 100) as f64;
            let max = ((i + 1) * 100) as f64;
            assert!(buckets[i].range == (min..max));
        }
    }

    #[test]
    fn find_slot_linear() {
        let options = HistogramOptions {
            intervals: 8,
            ..Default::default()
        };
        let hist = Histogram::new_with_stats(Stats::new(&[-12.0, 4.0], None), &options);
        assert!(hist.find_slot(-13.0) == None);
        assert!(hist.find_slot(13.0) == None);
        assert!(hist.find_slot(-12.0) == Some(0));
        assert!(hist.find_slot(-11.0) == Some(0));
        assert!(hist.find_slot(-9.0) == Some(1));
        assert!(hist.find_slot(4.0) == Some(7));
        assert!(hist.find_slot(1.1) == Some(6));
    }

    #[test]
    fn find_slot_logarithmic() {
        let hist = Histogram::new(
            // More than 8 values to avoid interval truncation
            &[255.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -2000.0],
            HistogramOptions {
                intervals: 8,
                log_scale: true,
                ..Default::default()
            },
        );
        assert!(hist.find_slot(-1.0) == None);
        assert!(hist.find_slot(0.0) == Some(0));
        assert!(hist.find_slot(0.5) == Some(0));
        assert!(hist.find_slot(1.5) == Some(1));
        assert!(hist.find_slot(8.75) == Some(3));
        assert!(hist.find_slot(33.1) == Some(5));
        assert!(hist.find_slot(127.1) == Some(7));
        assert!(hist.find_slot(247.1) == Some(7));
        assert!(hist.find_slot(1000.0) == None);
    }
}
