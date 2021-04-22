use std::fmt;

use yansi::Color::Blue;

#[derive(Debug)]
pub struct Stats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub std: f64,
    pub var: f64,
    pub sum: f64,
    pub samples: usize,
}

impl Stats {
    pub fn new(vec: &[f64]) -> Stats {
        let mut max = vec[0];
        let mut min = max;
        let mut temp: f64 = 0.0;
        let sum = vec.iter().sum::<f64>();
        let avg = sum / vec.len() as f64;
        for val in vec.iter() {
            max = max.max(*val);
            min = min.min(*val);
            temp += (avg - *val).powi(2)
        }
        let var = temp / vec.len() as f64;
        let std = var.sqrt();
        Stats {
            min,
            max,
            avg,
            std,
            var,
            sum,
            samples: vec.len(),
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Samples = {len:.5}; Min = {min:.5}; Max = {max:.5}",
            len = Blue.paint(self.samples.to_string()),
            min = Blue.paint(self.min.to_string()),
            max = Blue.paint(self.max.to_string()),
        )?;
        writeln!(
            f,
            "Average = {avg:.5}; Variance = {var:.5}; STD = {std:.5}",
            avg = Blue.paint(self.avg.to_string()),
            var = Blue.paint(self.var.to_string()),
            std = Blue.paint(self.std.to_string())
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
        let stats = Stats::new(&[1.1, 3.3, 2.2]);
        assert_eq!(3_usize, stats.samples);
        assert_float_eq!(stats.sum, 6.6, rmax <= f64::EPSILON);
        assert_float_eq!(stats.avg, 2.2, rmax <= f64::EPSILON);
        assert_float_eq!(stats.min, 1.1, rmax <= f64::EPSILON);
        assert_float_eq!(stats.max, 3.3, rmax <= f64::EPSILON);
        assert_float_eq!(stats.var, 0.8066, abs <= 0.0001);
        assert_float_eq!(stats.std, 0.8981, abs <= 0.0001);
    }

    #[test]
    fn test_display() {
        let stats = Stats::new(&[1.1, 3.3, 2.2]);
        Paint::disable();
        let display = format!("{}", stats);
        assert!(display.contains("Samples = 3"));
        assert!(display.contains("Min = 1.1"));
        assert!(display.contains("Max = 3.3"));
        assert!(display.contains("Average = 2.2"));
    }
}
