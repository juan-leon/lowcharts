use std::fmt;

use yansi::Color::{Blue, Green, Red};

#[derive(Debug)]
pub struct MatchBarRow {
    pub label: String,
    pub count: usize,
}

impl MatchBarRow {
    pub fn new(string: &str) -> MatchBarRow {
        MatchBarRow {
            label: string.to_string(),
            count: 0,
        }
    }

    pub fn inc_if_matches(&mut self, line: &str) {
        if line.contains(&self.label) {
            self.count += 1;
        }
    }
}

#[derive(Debug)]
pub struct MatchBar {
    pub vec: Vec<MatchBarRow>,
    top_values: usize,
    top_lenght: usize,
}

impl MatchBar {
    pub fn new(vec: Vec<MatchBarRow>) -> MatchBar {
        let mut top_lenght: usize = 0;
        let mut top_values: usize = 0;
        for row in vec.iter() {
            top_lenght = top_lenght.max(row.label.len());
            top_values = top_values.max(row.count);
        }
        MatchBar {
            vec,
            top_values,
            top_lenght,
        }
    }
}

impl fmt::Display for MatchBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(100);
        let divisor = 1.max(self.top_values / width);
        let width_count = format!("{}", self.top_values).len();
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
        for row in self.vec.iter() {
            writeln!(
                f,
                "[{label}] [{count}] {bar}",
                label = Blue.paint(format!("{:width$}", row.label, width = self.top_lenght)),
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
    fn test_matchbar() {
        let mut row0 = MatchBarRow::new("label1");
        row0.inc_if_matches("labelN");
        row0.inc_if_matches("label1");
        row0.inc_if_matches("label1");
        row0.inc_if_matches("label11");
        let mut row1 = MatchBarRow::new("label2");
        row1.inc_if_matches("label2");
        let mb = MatchBar::new(vec![row0, row1, MatchBarRow::new("label333")]);
        assert_eq!(mb.top_lenght, 8);
        assert_eq!(mb.top_values, 3);
        Paint::disable();
        let display = format!("{}", mb);

        assert!(display.contains("[label1  ] [3] ∎∎∎\n"));
        assert!(display.contains("[label2  ] [1] ∎\n"));
        assert!(display.contains("[label333] [0] \n"));
        assert!(display.contains("represents a count of 1"));
        assert!(display.contains("Matches: 4"));
    }
}
