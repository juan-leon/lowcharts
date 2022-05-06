use std::collections::HashMap;
use std::fmt;

use yansi::Color::{Blue, Green, Red};

#[derive(Debug)]
/// A struct holding data to plot a Histogram of the most frequent terms in an
/// arbitrary input.
///
/// The struct is create empty and it will fill its data by calling its
/// `observe` method.
pub struct CommonTerms {
    pub terms: HashMap<String, usize>,
    lines: usize,
}

impl CommonTerms {
    /// Create and empty `CommonTerms`.
    ///
    /// `lines` is the number of lines to be displayed.
    pub fn new(lines: usize) -> CommonTerms {
        CommonTerms {
            terms: HashMap::new(),
            lines,
        }
    }

    /// Observe a new "term".
    pub fn observe(&mut self, term: String) {
        *self.terms.entry(term).or_insert(0) += 1
    }
}

impl fmt::Display for CommonTerms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(100);
        let mut counts: Vec<(&String, &usize)> = self.terms.iter().collect();
        if counts.is_empty() {
            writeln!(f, "No data")?;
            return Ok(());
        }
        counts.sort_by(|a, b| b.1.cmp(a.1));
        let values = &counts[..self.lines.min(counts.len())];
        let label_width = values.iter().fold(1, |acc, x| acc.max(x.0.len()));
        let divisor = 1.max(counts[0].1 / width);
        let width_count = format!("{}", counts[0].1).len();
        writeln!(
            f,
            "Each {} represents a count of {}",
            Red.paint("∎"),
            Blue.paint(divisor.to_string()),
        )?;
        for (term, count) in values.iter() {
            writeln!(
                f,
                "[{label}] [{count}] {bar}",
                label = Blue.paint(format!("{:>width$}", term, width = label_width)),
                count = Green.paint(format!("{:width$}", count, width = width_count)),
                bar = Red.paint(format!("{:∎<width$}", "", width = *count / divisor))
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
    fn test_common_terms_empty() {
        let terms = CommonTerms::new(10);
        Paint::disable();
        let display = format!("{}", terms);
        assert_eq!(display, "No data\n");
    }

    #[test]
    fn test_common_terms() {
        let mut terms = CommonTerms::new(2);
        for _ in 0..100 {
            terms.observe(String::from("foo"));
        }
        for _ in 0..10 {
            terms.observe(String::from("arrrrrrrr"));
        }
        for _ in 0..20 {
            terms.observe(String::from("barbar"));
        }
        Paint::disable();
        let display = format!("{:10}", terms);

        println!("{}", display);
        assert!(display.contains("[   foo] [100] ∎∎∎∎∎∎∎∎∎∎\n"));
        assert!(display.contains("[barbar] [ 20] ∎∎\n"));
        assert!(!display.contains("arr"));
    }
}
