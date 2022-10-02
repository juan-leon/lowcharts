pub use self::histogram::{Histogram, HistogramOptions};
pub use self::matchbar::{MatchBar, MatchBarRow};
pub use self::splittimehist::SplitTimeHistogram;
pub use self::terms::CommonTerms;
pub use self::timehist::TimeHistogram;
pub use self::xy::XyPlot;

mod histogram;
mod matchbar;
mod splittimehist;
mod terms;
mod timehist;
mod xy;

/// Returns a datetime formating string with a resolution that makes sense for a
/// given number of seconds
fn date_fmt_string(seconds: i64) -> &'static str {
    match seconds {
        x if x > 86400 => "%Y-%m-%d %H:%M:%S",
        x if x > 300 => "%H:%M:%S",
        x if x > 1 => "%H:%M:%S%.3f",
        _ => "%H:%M:%S%.6f",
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fmt_strings() {
        assert_eq!(date_fmt_string(100000), "%Y-%m-%d %H:%M:%S");
        assert_eq!(date_fmt_string(1000), "%H:%M:%S");
        assert_eq!(date_fmt_string(10), "%H:%M:%S%.3f");
        assert_eq!(date_fmt_string(0), "%H:%M:%S%.6f");
    }
}
