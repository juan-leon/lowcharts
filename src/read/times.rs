use std::io::BufRead;

use chrono::{DateTime, Duration, FixedOffset};
use regex::Regex;

use crate::read::dateparser::LogDateParser;
use crate::read::open_file;

#[derive(Default, Builder)]
pub struct TimeReader {
    #[builder(setter(strip_option), default)]
    regex: Option<Regex>,
    #[builder(setter(strip_option), default)]
    ts_format: Option<String>,
    #[builder(setter(strip_option), default)]
    duration: Option<Duration>,
    #[builder(default)]
    early_stop: bool,
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
        let cut_datetime = match self.duration {
            Some(duration) => {
                if !self.early_stop || vec.is_empty() {
                    None
                } else {
                    Some(vec[0] + duration)
                }
            }
            _ => None,
        };
        for line in iterator {
            match line {
                Ok(string) => {
                    if let Ok(x) = parser.parse(&string) {
                        match cut_datetime {
                            Some(d) if x > d => break,
                            _ => (),
                        };
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
        if cut_datetime.is_none() {
            if let Some(duration) = self.duration {
                if let Some(min) = vec.iter().min() {
                    let max = *min + duration;
                    vec.retain(|&d| d <= max);
                }
            }
        }
        vec
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn time_reader_guessing_with_regex() {
        let mut builder = TimeReaderBuilder::default();
        builder.regex(Regex::new("f.o").unwrap());
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
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

    #[test]
    fn time_reader_with_format() {
        let mut builder = TimeReaderBuilder::default();
        builder.ts_format(String::from("%Y_%m_%d %H:%M"));
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
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

    #[test]
    fn time_with_duration() {
        let mut builder = TimeReaderBuilder::default();
        builder.duration(Duration::seconds(90));
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[2021-04-15T06:25:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:26:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:27:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:28:31+00:00] foo").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 2);
        assert_eq!(
            ts[0],
            DateTime::parse_from_rfc3339("2021-04-15T06:25:31+00:00").unwrap()
        );
        assert_eq!(
            ts[1],
            DateTime::parse_from_rfc3339("2021-04-15T06:26:31+00:00").unwrap()
        );
    }

    #[test]
    fn time_with_early_stop() {
        let mut builder = TimeReaderBuilder::default();
        builder.duration(Duration::seconds(90)).early_stop(true);
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[2021-04-15T06:25:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:26:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:27:31+00:00] foo").unwrap();
        // This date goes backwards
        writeln!(file, "[2021-04-15T06:25:32+00:00] foo").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 2);
        assert_eq!(
            ts[0],
            DateTime::parse_from_rfc3339("2021-04-15T06:25:31+00:00").unwrap()
        );
        assert_eq!(
            ts[1],
            DateTime::parse_from_rfc3339("2021-04-15T06:26:31+00:00").unwrap()
        );
    }

    #[test]
    fn time_reader_with_bad_format() {
        let mut builder = TimeReaderBuilder::default();
        builder.ts_format(String::from("%Y_%zzzz"));
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "_2021_04_15 06:25] foobar").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn time_empty_file() {
        let reader = TimeReaderBuilder::default().build().unwrap();
        let file = NamedTempFile::new().unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn time_reader_with_garbage() {
        let reader = TimeReaderBuilder::default().build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "garbage").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }
}
