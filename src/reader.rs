use std::io::{self, BufRead};
use std::fs::File;
use std::ops::Range;

use regex::Regex;
use yansi::Color::{Red, Magenta};


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

    pub fn read(&self, path: String) -> Vec<f64> {
        let mut vec: Vec<f64> = vec![];
        match path.as_str() {
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
            None => Self::parse_float
        };
        for line in lines {
            match line {
                Ok(as_string) => if let Some(n) = line_parser(&self, &as_string) {
                    match &self.range {
                        Some(range) => if range.contains(&n) {
                            vec.push(n);
                        },
                        _ => vec.push(n)
                    }
                },
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
            },
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
