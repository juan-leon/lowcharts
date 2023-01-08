use std::fmt;

use yansi::Color::Blue;

use crate::format::F64Formatter;

#[derive(Debug)]
/// A struct holding statistical data regarding a unsorted set of numerical
/// values.
pub struct Stats {
    /// Minimum of the input values.
    pub min: f64,
    /// Maximum of the input values.
    pub max: f64,
    /// Average of the input values.
    pub avg: f64,
    /// Standard deviation of the input values.
    pub std: f64,
    /// Variance of the input values.
    pub var: f64,
    /// Number of samples of the input values.
    pub samples: usize,
    precision: Option<usize>, // If None, then human friendly display will be used
}

impl Stats {
    /// Creates a Stats struct from a vector of numerical data.
    ///
    /// `precision` is an Option with the number of decimals to display.  If
    /// "None" is used, human units will be used, with an heuristic based on the
    /// input data for deciding the units and the decimal places.
    pub fn new(vec: &[f64], precision: Option<usize>) -> Self {
        let mut max = vec[0];
        let mut min = max;
        let mut temp: f64 = 0.0;
        let sum = vec.iter().sum::<f64>();
        let avg = sum / vec.len() as f64;
        for val in vec.iter() {
            max = max.max(*val);
            min = min.min(*val);
            temp += (avg - *val).powi(2);
        }
        let var = temp / vec.len() as f64;
        let std = var.sqrt();
        Self {
            min,
            max,
            avg,
            std,
            var,
            samples: vec.len(),
            precision,
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatter = match self.precision {
            None => F64Formatter::new_with_range(self.min..self.max),
            Some(n) => F64Formatter::new(n),
        };
        writeln!(
            f,
            "Samples = {len}; Min = {min}; Max = {max}",
            len = Blue.paint(self.samples.to_string()),
            min = Blue.paint(formatter.format(self.min)),
            max = Blue.paint(formatter.format(self.max)),
        )?;
        writeln!(
            f,
            "Average = {avg}; Variance = {var}; STD = {std}",
            avg = Blue.paint(formatter.format(self.avg)),
            var = Blue.paint(format!("{:.3}", self.var)),
            std = Blue.paint(format!("{:.3}", self.std)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;
    use yansi::Paint;

    #[test]
    fn basic_test() {
        let stats = Stats::new(&[1.1, 3.3, 2.2], Some(3));
        assert_eq!(3_usize, stats.samples);
        assert_float_eq!(stats.avg, 2.2, rmax <= f64::EPSILON);
        assert_float_eq!(stats.min, 1.1, rmax <= f64::EPSILON);
        assert_float_eq!(stats.max, 3.3, rmax <= f64::EPSILON);
        assert_float_eq!(stats.var, 0.8066, abs <= 0.0001);
        assert_float_eq!(stats.std, 0.8981, abs <= 0.0001);
    }

    #[test]
    fn test_display() {
        let stats = Stats::new(&[1.1, 3.3, 2.2], Some(3));
        Paint::disable();
        let display = format!("{stats}");
        assert!(display.contains("Samples = 3"));
        assert!(display.contains("Min = 1.100"));
        assert!(display.contains("Max = 3.300"));
        assert!(display.contains("Average = 2.200"));
    }

    #[test]
    fn test_big_num() {
        let stats = Stats::new(&[123456789.1234, 123456788.1234], None);
        Paint::disable();
        let display = format!("{stats}");
        assert!(display.contains("Samples = 2"));
        assert!(display.contains("Min = 123456788.123"));
        assert!(display.contains("Max = 123456789.123"));
    }
}
