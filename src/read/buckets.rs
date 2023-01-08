use std::io::BufRead;
use std::ops::Range;

use regex::Regex;

use crate::plot::{CommonTerms, MatchBar, MatchBarRow};
use crate::read::open_file;

#[derive(Debug, Default, Builder)]
pub struct DataReader {
    #[builder(setter(strip_option), default)]
    range: Option<Range<f64>>,
    #[builder(setter(strip_option), default)]
    regex: Option<Regex>,
}

impl DataReader {
    pub fn read(&self, path: &str) -> Vec<f64> {
        let mut vec: Vec<f64> = Vec::new();
        let line_parser = match self.regex {
            Some(_) => Self::parse_regex,
            None => Self::parse_float,
        };
        for line in open_file(path).lines() {
            match line {
                Ok(as_string) => {
                    if let Some(n) = line_parser(self, &as_string) {
                        match &self.range {
                            Some(range) => {
                                if range.contains(&n) {
                                    vec.push(n);
                                }
                            }
                            _ => vec.push(n),
                        }
                    }
                }
                Err(error) => error!("{}", error),
            }
        }
        vec
    }

    fn parse_float(&self, line: &str) -> Option<f64> {
        match line.parse::<f64>() {
            Ok(n) => Some(n),
            Err(parse_error) => {
                debug!("Cannot parse float ({}) at '{}'", parse_error, line);
                None
            }
        }
    }

    fn parse_regex(&self, line: &str) -> Option<f64> {
        match self.regex.as_ref().unwrap().captures(line) {
            Some(cap) => {
                if let Some(name) = cap.name("value") {
                    self.parse_float(name.as_str())
                } else if let Some(capture) = cap.get(1) {
                    self.parse_float(capture.as_str())
                } else {
                    None
                }
            }
            None => {
                debug!("Regex does not match '{}'", line);
                None
            }
        }
    }

    pub fn read_matches(&self, path: &str, strings: Vec<&str>) -> MatchBar {
        let mut rows = Vec::<MatchBarRow>::with_capacity(strings.len());
        for s in strings {
            rows.push(MatchBarRow::new(s));
        }
        for line in open_file(path).lines() {
            match line {
                Ok(as_string) => {
                    for row in rows.iter_mut() {
                        row.inc_if_matches(&as_string);
                    }
                }
                Err(error) => error!("{}", error),
            }
        }
        MatchBar::new(rows)
    }

    pub fn read_terms(&self, path: &str, lines: usize) -> CommonTerms {
        let mut terms = CommonTerms::new(lines);
        let regex = self.regex.as_ref().unwrap();
        for line in open_file(path).lines() {
            match line {
                Ok(as_string) => {
                    if let Some(cap) = regex.captures(&as_string) {
                        if let Some(name) = cap.name("value") {
                            terms.observe(String::from(name.as_str()));
                        } else if let Some(capture) = cap.get(1) {
                            terms.observe(String::from(capture.as_str()));
                        }
                    };
                }
                Err(error) => error!("{}", error),
            }
        }
        terms
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn basic_reader_test() {
        let reader = DataReader::default();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1.3").unwrap();
        writeln!(file, "foobar").unwrap();
        writeln!(file, "2").unwrap();
        writeln!(file, "-2.7").unwrap();
        let vec = reader.read(file.path().to_str().unwrap());
        assert_eq!(vec, [1.3, 2.0, -2.7]);
    }

    #[test]
    fn regex_first_match() {
        let re = Regex::new("^foo ([0-9.-]+) ([0-9.-]+)").unwrap();
        let reader = DataReaderBuilder::default().regex(re).build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "foo 1.3 1.6").unwrap();
        writeln!(file, "nothing").unwrap();
        writeln!(file, "1.1").unwrap();
        writeln!(file, "1.1 1.2").unwrap();
        writeln!(file, "foo -2 3").unwrap();
        writeln!(file, "foo 5").unwrap();
        let vec = reader.read(file.path().to_str().unwrap());
        assert_eq!(vec, [1.3, -2.0]);
    }

    #[test]
    fn regex_named_match() {
        let re = Regex::new("^foo ([0-9.-]+) (?P<value>[0-9.-]+)").unwrap();
        let reader = DataReaderBuilder::default().regex(re).build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "foo 1.3 1.6").unwrap();
        writeln!(file, "nothing").unwrap();
        writeln!(file, "1.1").unwrap();
        writeln!(file, "1.1 1.2").unwrap();
        writeln!(file, "foo -2 3").unwrap();
        writeln!(file, "foo 5").unwrap();
        let vec = reader.read(file.path().to_str().unwrap());
        assert_eq!(vec, [1.6, 3.0]);
    }

    #[test]
    fn regex_empty_file() {
        let reader = DataReader::default();
        let file = NamedTempFile::new().unwrap();
        let vec = reader.read(file.path().to_str().unwrap());
        assert_eq!(vec, []);
    }

    #[test]
    fn range() {
        let reader = DataReaderBuilder::default()
            .range(-1.0..1.0)
            .build()
            .unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1.3").unwrap();
        writeln!(file, "2").unwrap();
        writeln!(file, "-0.5").unwrap();
        writeln!(file, "0.5").unwrap();
        let vec = reader.read(file.path().to_str().unwrap());
        assert_eq!(vec, [-0.5, 0.5]);
    }

    #[test]
    fn basic_match_reader() {
        let reader = DataReader::default();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "foobar").unwrap();
        writeln!(file, "data data foobar").unwrap();
        writeln!(file, "data data").unwrap();
        writeln!(file, "foobar").unwrap();
        writeln!(file, "none").unwrap();
        let mb = reader.read_matches(
            file.path().to_str().unwrap(),
            vec!["random", "foobar", "data"],
        );
        assert_eq!(mb.vec[0].label, "random");
        assert_eq!(mb.vec[0].count, 0);
        assert_eq!(mb.vec[1].label, "foobar");
        assert_eq!(mb.vec[1].count, 3);
        assert_eq!(mb.vec[2].label, "data");
        assert_eq!(mb.vec[2].count, 2);
    }

    #[test]
    fn basic_term_reader() {
        let re = Regex::new("^foo ([0-9.-]+) (?P<value>[0-9.-]+)").unwrap();
        let reader = DataReaderBuilder::default().regex(re).build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "foo 1.1 1.6").unwrap();
        writeln!(file, "foo 1.2 1.5").unwrap();
        writeln!(file, "foo 1.3 1.6").unwrap();
        writeln!(file, "foo 1.4 1.7").unwrap();
        let ct = reader.read_terms(file.path().to_str().unwrap(), 10);
        assert_eq!(ct.terms.len(), 3);
        assert_eq!(*ct.terms.get(&String::from("1.5")).unwrap(), 1);
        assert_eq!(*ct.terms.get(&String::from("1.6")).unwrap(), 2);
        assert_eq!(*ct.terms.get(&String::from("1.7")).unwrap(), 1);
        // Now, with no named capture group
        let re = Regex::new("^foo ([0-9.-]+) ([0-9.-]+)").unwrap();
        let reader = DataReaderBuilder::default().regex(re).build().unwrap();
        let ct = reader.read_terms(file.path().to_str().unwrap(), 10);
        assert_eq!(ct.terms.len(), 4);
        assert_eq!(*ct.terms.get(&String::from("1.1")).unwrap(), 1);
        assert_eq!(*ct.terms.get(&String::from("1.2")).unwrap(), 1);
        assert_eq!(*ct.terms.get(&String::from("1.3")).unwrap(), 1);
        assert_eq!(*ct.terms.get(&String::from("1.4")).unwrap(), 1);
    }
}
