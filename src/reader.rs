use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::Range;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use crate::dateparser::LogDateParser;
use crate::matchbar::{MatchBar, MatchBarRow};

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
                    if let Some(n) = line_parser(&self, &as_string) {
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
                    self.parse_float(&name.as_str())
                } else if let Some(capture) = cap.get(1) {
                    self.parse_float(&capture.as_str())
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
}

#[derive(Default, Builder)]
pub struct TimeReader {
    #[builder(setter(strip_option), default)]
    regex: Option<Regex>,
    #[builder(setter(strip_option), default)]
    ts_format: Option<String>,
}

impl TimeReader {
    pub fn read(&self, path: &str) -> Vec<DateTime<FixedOffset>> {
        let mut vec: Vec<DateTime<FixedOffset>> = Vec::new();
        let mut iterator = open_file(path).lines();
        let first_line = match iterator.next() {
            Some(Ok(as_string)) => as_string,
            Some(Err(error)) => {
                error!("{}", error);
                return vec;
            }
            _ => return vec,
        };
        let parser = match &self.ts_format {
            Some(ts_format) => match LogDateParser::new_with_format(&first_line, &ts_format) {
                Ok(p) => p,
                Err(error) => {
                    error!("Could not figure out parsing strategy: {}", error);
                    return vec;
                }
            },
            None => match LogDateParser::new_with_guess(&first_line) {
                Ok(p) => p,
                Err(error) => {
                    error!("Could not figure out parsing strategy: {}", error);
                    return vec;
                }
            },
        };
        if let Ok(x) = parser.parse(&first_line) {
            vec.push(x);
        }
        for line in iterator {
            match line {
                Ok(string) => {
                    if let Ok(x) = parser.parse(&string) {
                        if let Some(re) = &self.regex {
                            if re.is_match(&string) {
                                vec.push(x);
                            }
                        } else {
                            vec.push(x);
                        }
                    }
                }
                Err(error) => error!("{}", error),
            }
        }
        vec
    }
}

fn open_file(path: &str) -> Box<dyn io::BufRead> {
    match path {
        "-" => Box::new(BufReader::new(io::stdin())),
        _ => match File::open(path) {
            Ok(fd) => Box::new(io::BufReader::new(fd)),
            Err(error) => {
                error!("Could not open {}: {}", path, error);
                std::process::exit(0);
            }
        },
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
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                writeln!(file, "1.3").unwrap();
                writeln!(file, "foobar").unwrap();
                writeln!(file, "2").unwrap();
                writeln!(file, "-2.7").unwrap();
                let vec = reader.read(file.path().to_str().unwrap());
                assert_eq!(vec, [1.3, 2.0, -2.7]);
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn regex_first_match() {
        let re = Regex::new("^foo ([0-9.-]+) ([0-9.-]+)").unwrap();
        let reader = DataReaderBuilder::default().regex(re).build().unwrap();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                writeln!(file, "foo 1.3 1.6").unwrap();
                writeln!(file, "nothing").unwrap();
                writeln!(file, "1.1").unwrap();
                writeln!(file, "1.1 1.2").unwrap();
                writeln!(file, "foo -2 3").unwrap();
                writeln!(file, "foo 5").unwrap();
                let vec = reader.read(file.path().to_str().unwrap());
                assert_eq!(vec, [1.3, -2.0]);
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn regex_named_match() {
        let re = Regex::new("^foo ([0-9.-]+) (?P<value>[0-9.-]+)").unwrap();
        let reader = DataReaderBuilder::default().regex(re).build().unwrap();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                writeln!(file, "foo 1.3 1.6").unwrap();
                writeln!(file, "nothing").unwrap();
                writeln!(file, "1.1").unwrap();
                writeln!(file, "1.1 1.2").unwrap();
                writeln!(file, "foo -2 3").unwrap();
                writeln!(file, "foo 5").unwrap();
                let vec = reader.read(file.path().to_str().unwrap());
                assert_eq!(vec, [1.6, 3.0]);
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn regex_empty_file() {
        let reader = DataReader::default();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                let vec = reader.read(file.path().to_str().unwrap());
                assert_eq!(vec, []);
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn range() {
        let reader = DataReaderBuilder::default()
            .range(-1.0..1.0)
            .build()
            .unwrap();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                writeln!(file, "1.3").unwrap();
                writeln!(file, "2").unwrap();
                writeln!(file, "-0.5").unwrap();
                writeln!(file, "0.5").unwrap();
                let vec = reader.read(file.path().to_str().unwrap());
                assert_eq!(vec, [-0.5, 0.5]);
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn basic_match_reader() {
        let reader = DataReader::default();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
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
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn time_reader_guessing_with_regex() {
        let mut builder = TimeReaderBuilder::default();
        builder.regex(Regex::new("f.o").unwrap());
        let reader = builder.build().unwrap();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                writeln!(file, "[2021-04-15T06:25:31+00:00] foobar").unwrap();
                writeln!(file, "[2021-04-15T06:26:31+00:00] bar").unwrap();
                writeln!(file, "[2021-04-15T06:27:31+00:00] foobar").unwrap();
                writeln!(file, "[2021-04-15T06:28:31+00:00] foobar").unwrap();
                writeln!(file, "none").unwrap();
                let ts = reader.read(file.path().to_str().unwrap());
                assert_eq!(ts.len(), 3);
                assert_eq!(
                    ts[0],
                    DateTime::parse_from_rfc3339("2021-04-15T06:25:31+00:00").unwrap()
                );
                assert_eq!(
                    ts[2],
                    DateTime::parse_from_rfc3339("2021-04-15T06:28:31+00:00").unwrap()
                );
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }

    #[test]
    fn time_reader_with_format() {
        let mut builder = TimeReaderBuilder::default();
        builder.ts_format(String::from("%Y_%m_%d %H:%M"));
        let reader = builder.build().unwrap();
        match NamedTempFile::new() {
            Ok(ref mut file) => {
                writeln!(file, "_2021_04_15 06:25] foobar").unwrap();
                writeln!(file, "_2021_04_15 06:26] bar").unwrap();
                writeln!(file, "_2021_04_15 06:27] foobar").unwrap();
                writeln!(file, "_2021_04_15 06:28] foobar").unwrap();
                writeln!(file, "none").unwrap();
                let ts = reader.read(file.path().to_str().unwrap());
                assert_eq!(ts.len(), 4);
                assert_eq!(
                    ts[0],
                    DateTime::parse_from_rfc3339("2021-04-15T06:25:00+00:00").unwrap()
                );
                assert_eq!(
                    ts[3],
                    DateTime::parse_from_rfc3339("2021-04-15T06:28:00+00:00").unwrap()
                );
            }
            Err(_) => assert!(false, "Could not create temp file"),
        }
    }
}
