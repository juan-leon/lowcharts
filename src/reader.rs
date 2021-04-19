use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Range;

use regex::Regex;
use yansi::Color::{Magenta, Red};

#[derive(Debug, Default, Builder)]
pub struct DataReader {
    #[builder(setter(strip_option), default)]
    range: Option<Range<f64>>,
    #[builder(setter(strip_option), default)]
    regex: Option<Regex>,
    #[builder(default)]
    verbose: bool,
}

impl DataReader {
    pub fn read(&self, path: &str) -> Vec<f64> {
        let mut vec: Vec<f64> = vec![];
        match path {
            "-" => {
                vec = self.read_data(io::stdin().lock().lines());
            }
            _ => {
                let file = File::open(path);
                match file {
                    Ok(fd) => {
                        vec = self.read_data(io::BufReader::new(fd).lines());
                    }
                    Err(error) => eprintln!("[{}]: {}", Red.paint("ERROR"), error),
                }
            }
        }
        vec
    }

    fn read_data<T: BufRead>(&self, lines: std::io::Lines<T>) -> Vec<f64> {
        let mut vec: Vec<f64> = Vec::new();
        let line_parser = match self.regex {
            Some(_) => Self::parse_regex,
            None => Self::parse_float,
        };
        for line in lines {
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
                Err(error) => eprintln!("[{}]: {}", Red.paint("ERROR"), error),
            }
        }
        vec
    }

    fn parse_float(&self, line: &str) -> Option<f64> {
        match line.parse::<f64>() {
            Ok(n) => Some(n),
            Err(parse_error) => {
                eprintln!(
                    "[{}] Cannot parse float ({}) at '{}'",
                    Red.paint("ERROR"),
                    parse_error,
                    line
                );
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
                if self.verbose {
                    eprintln!(
                        "[{}] Regex does not match '{}'",
                        Magenta.paint("DEBUG"),
                        line
                    );
                }
                None
            }
        }
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
}
