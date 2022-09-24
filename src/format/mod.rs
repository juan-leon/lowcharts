use std::fmt;
use std::ops::Range;

use yansi::Color::{Blue, Green, Red};
use yansi::Paint;

// Units-based suffixes for human formatting.
const UNITS: &[&str] = &["", " K", " M", " G", " T", " P", " E", " Z", " Y"];
pub static BAR_CHAR: &str = "∎";

#[derive(Debug)]
pub struct F64Formatter {
    /// Decimals digits to be used
    decimals: usize,
    /// Number of times the value will be divided by 1000
    divisor: u8,
    /// Suffix (typycally units) to be printed after number
    suffix: String,
}

impl F64Formatter {
    /// Initializes a new `HumanF64Formatter` with default values.
    pub fn new(decimals: usize) -> F64Formatter {
        F64Formatter {
            decimals,
            divisor: 0,
            suffix: "".to_owned(),
        }
    }

    /// Initializes a new `HumanF64Formatter` for formatting numbers in the
    /// provided range.
    pub fn new_with_range(range: Range<f64>) -> F64Formatter {
        // Range
        let mut decimals = 3;
        let mut divisor = 0_u8;
        let mut suffix = UNITS[0].to_owned();
        let difference = range.end - range.start;
        if difference == 0.0 {
            return F64Formatter {
                decimals,
                divisor,
                suffix,
            };
        }
        let log = difference.abs().log10() as i64;
        if log <= 0 {
            decimals = (-log as usize).min(8) + 3;
        } else {
            decimals = log.rem_euclid(3) as usize;
            divisor = ((log - 1) / 3).min(5) as u8;
        }
        suffix = UNITS[divisor as usize].to_owned();
        F64Formatter {
            decimals,
            divisor,
            suffix,
        }
    }

    pub fn format(&self, number: f64) -> String {
        format!(
            "{:.*}{}",
            self.decimals,
            number / 1000_usize.pow(self.divisor.into()) as f64,
            self.suffix
        )
    }
}

pub struct HorizontalScale {
    /// How many units are represented by a char
    scale: usize,
}

impl HorizontalScale {
    pub fn new(scale: usize) -> HorizontalScale {
        HorizontalScale {
            scale: 1.max(scale),
        }
    }

    pub fn get_bar(&self, units: usize) -> Paint<String> {
        Red.paint(format!("{:∎<width$}", "", width = units / self.scale))
    }

    pub fn get_count(&self, units: usize, width: usize) -> Paint<String> {
        Green.paint(format!("{:width$}", units, width = width))
    }

    pub fn get_scale(&self) -> usize {
        self.scale
    }
}

impl fmt::Display for HorizontalScale {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Each {} represents a count of {}",
            Red.paint(BAR_CHAR),
            Blue.paint(self.scale.to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yansi::Paint;

    #[test]
    fn test_basic_format() {
        assert_eq!(F64Formatter::new(0).format(1000.0), "1000");
        assert_eq!(F64Formatter::new(3).format(1000.0), "1000.000");
        assert_eq!(F64Formatter::new(1).format(12345.299), "12345.3");
        assert_eq!(F64Formatter::new(10).format(3.0), "3.0000000000");
    }

    #[test]
    fn test_human_format_from_zero() {
        assert_eq!(F64Formatter::new_with_range(0.0..2.0).format(1.12), "1.120");
        assert_eq!(
            F64Formatter::new_with_range(0.0..200.0).format(234.12),
            "234.12"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..1000.0).format(234.1234),
            "234"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..10000.0).format(234.1234),
            "0.2 K"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..100000.0).format(234.1234),
            "0.23 K"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..1000000.0).format(234.1234),
            "0 K"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..100000000.0).format(1234.1234),
            "0.00 M"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..1000000.0).format(234000.1234),
            "234 K"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..100000000.0).format(1234000.1234),
            "1.23 M"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..100000000.0).format(12340000.1234),
            "12.34 M"
        );
    }

    #[test]
    fn test_human_format_small_numbers() {
        assert_eq!(
            F64Formatter::new_with_range(0.0..0.0002).format(0.0000043),
            "0.000004"
        );
        assert_eq!(
            F64Formatter::new_with_range(0.0..0.00002).format(0.0000043),
            "0.0000043"
        );
        assert_eq!(
            F64Formatter::new_with_range(20000.0..20000.00002).format(20000.0000043),
            "20000.0000043"
        );
    }

    #[test]
    fn test_human_format_bignum_small_interval() {
        assert_eq!(
            F64Formatter::new_with_range(100000000.0..100000001.0).format(100000000.12341234),
            "100000000.123"
        );
    }

    #[test]
    fn test_human_format_negative_start() {
        assert_eq!(
            F64Formatter::new_with_range(-4.0..2.0).format(1.12),
            "1.120"
        );
        assert_eq!(
            F64Formatter::new_with_range(-4.0..-2.0).format(-3.12),
            "-3.120"
        );
        assert_eq!(
            F64Formatter::new_with_range(-10000000.0..10.0).format(-3.12),
            "-0.0 M"
        );
    }

    #[test]
    fn test_horizontal_scale() {
        Paint::disable();
        assert_eq!(
            format!("{}", HorizontalScale::new(123)),
            format!("Each {BAR_CHAR} represents a count of 123\n")
        );
    }

    #[test]
    fn test_horizontal_scale_with_zero_scale() {
        Paint::disable();
        let scale = HorizontalScale::new(0);
        assert_eq!(scale.get_scale(), 1);
        assert_eq!(
            format!("{}", scale),
            format!("Each {BAR_CHAR} represents a count of 1\n")
        );
    }

    #[test]
    fn test_horizontal_scale_bar() {
        let scale = HorizontalScale::new(10);
        assert_eq!(
            scale.get_bar(80),
            Red.paint(format!("{:∎<width$}", "", width = 8))
        );
    }

    #[test]
    fn test_horizontal_scale_count() {
        let scale = HorizontalScale::new(10);
        assert_eq!(scale.get_count(80, 5), Green.paint(format!("   80")));
    }
}
