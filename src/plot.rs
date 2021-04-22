use std::fmt;
use std::ops::Range;

use yansi::Color::{Blue, Red};

use crate::stats::Stats;

#[derive(Debug)]
pub struct Plot {
    x_axis: Vec<f64>,
    y_axis: Vec<f64>,
    width: usize,
    height: usize,
    stats: Stats,
}

impl Plot {
    pub fn new(width: usize, height: usize, stats: Stats) -> Plot {
        Plot {
            x_axis: Vec::with_capacity(width),
            y_axis: Vec::with_capacity(height),
            width,
            height,
            stats,
        }
    }

    pub fn load(&mut self, vec: &[f64]) {
        self.width = self.width.min(vec.len());
        let num_chunks = vec.len() / self.width;
        let iter = vec.chunks(num_chunks);
        for x in iter {
            let sum: f64 = x.iter().sum();
            self.x_axis.push(sum / x.len() as f64);
        }
        let step = (self.stats.max - self.stats.min) / self.height as f64;
        for y in 0..self.height {
            self.y_axis.push(self.stats.min + step * y as f64);
        }
    }
}

impl fmt::Display for Plot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stats)?;
        let _step = (self.stats.max - self.stats.min) / self.height as f64;
        let y_width = format!("{:.3}", self.stats.max).len();
        let mut newvec = self.y_axis.to_vec();
        newvec.reverse();
        print_line(f, &self.x_axis, newvec[0]..f64::INFINITY, y_width)?;
        for y in newvec.windows(2) {
            print_line(f, &self.x_axis, y[1]..y[0], y_width)?;
        }
        Ok(())
    }
}

fn print_line(
    f: &mut fmt::Formatter,
    x_axis: &[f64],
    range: Range<f64>,
    y_width: usize,
) -> fmt::Result {
    let mut row = format!("{: <width$}", "", width = x_axis.len());
    // The reverse in the enumeration is to avoid breaking char boundaries
    // because of unicode char ● having more bytes than ascii chars.
    for (x, value) in x_axis.iter().enumerate().rev() {
        if range.contains(value) {
            row.replace_range(x..x + 1, "●".as_ref());
        }
    }
    writeln!(
        f,
        "[{}] {}",
        Blue.paint(format!("{y:.*}", y_width, y = range.start.to_string())),
        Red.paint(row),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;
    use yansi::Paint;

    #[test]
    fn basic_test() {
        let stats = Stats::new(&[-1.0, 4.0]);
        let mut plot = Plot::new(3, 5, stats);
        plot.load(&[-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, -1.0]);
        assert_float_eq!(plot.x_axis[0], -0.5, rmax <= f64::EPSILON);
        assert_float_eq!(plot.x_axis[1], 1.5, rmax <= f64::EPSILON);
        assert_float_eq!(plot.x_axis[2], 3.5, rmax <= f64::EPSILON);
        assert_float_eq!(plot.x_axis[3], -1.0, rmax <= f64::EPSILON);

        assert_float_eq!(plot.y_axis[0], -1.0, rmax <= f64::EPSILON);
        assert_float_eq!(plot.y_axis[4], 3.0, rmax <= f64::EPSILON);
    }

    #[test]
    fn display_test() {
        let stats = Stats::new(&[-1.0, 4.0]);
        let mut plot = Plot::new(3, 5, stats);
        plot.load(&[-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, -1.0]);
        Paint::disable();
        let display = format!("{}", plot);
        assert!(display.contains("[3]   ● "));
        assert!(display.contains("[2]     "));
        assert!(display.contains("[1]  ●  "));
        assert!(display.contains("[-1] ●  ●"));
    }
}
