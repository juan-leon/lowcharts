use std::io::BufRead;

use chrono::{DateTime, FixedOffset};

use crate::read::dateparser::LogDateParser;
use crate::read::open_file;

#[derive(Default, Builder)]
pub struct SplitTimeReader {
    #[builder(setter(strip_option), default)]
    matches: Vec<String>,
    #[builder(setter(strip_option), default)]
    ts_format: Option<String>,
}

impl SplitTimeReader {
    pub fn read(&self, path: &str) -> Vec<(DateTime<FixedOffset>, usize)> {
        let mut vec: Vec<(DateTime<FixedOffset>, usize)> = Vec::new();
        let mut iterator = open_file(path).lines();
        let first_line = match iterator.next() {
            Some(Ok(as_string)) => as_string,
            Some(Err(error)) => {
                error!("{}", error);
                return vec;
            }
            _ => return vec,
        };
        let parser = match LogDateParser::new(&first_line, &self.ts_format) {
            Ok(p) => p,
            Err(error) => {
                error!("Could not figure out parsing strategy: {}", error);
                return vec;
            }
        };
        if let Ok(x) = parser.parse(&first_line) {
            self.push_conditionally(x, &mut vec, &first_line);
        }
        for line in iterator {
            match line {
                Ok(string) => {
                    if let Ok(x) = parser.parse(&string) {
                        self.push_conditionally(x, &mut vec, &string);
                    }
                }
                Err(error) => error!("{}", error),
            }
        }
        vec
    }

    fn push_conditionally(
        &self,
        d: DateTime<FixedOffset>,
        vec: &mut Vec<(DateTime<FixedOffset>, usize)>,
        line: &str,
    ) {
        for (i, s) in self.matches.iter().enumerate() {
            if line.contains(s) {
                vec.push((d, i));
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
    fn split_time_reader_basic() {
        let mut builder = SplitTimeReaderBuilder::default();
        builder.matches(vec![
            "foo".to_string(),
            "bar".to_string(),
            "gnat".to_string(),
        ]);
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[2021-04-15T06:25:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:26:31+00:00] bar").unwrap();
        writeln!(file, "[2021-04-15T06:27:31+00:00] foobar").unwrap();
        writeln!(file, "[2021-04-15T06:28:31+00:00] none").unwrap();
        writeln!(file, "[2021-04-15T06:29:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:30:31+00:00] none again").unwrap();
        writeln!(file, "not even a timestamp").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 5);
        assert_eq!(
            ts[0].0,
            DateTime::parse_from_rfc3339("2021-04-15T06:25:31+00:00").unwrap()
        );
        assert_eq!(
            ts[1].0,
            DateTime::parse_from_rfc3339("2021-04-15T06:26:31+00:00").unwrap()
        );
        assert_eq!(
            ts[2].0,
            DateTime::parse_from_rfc3339("2021-04-15T06:27:31+00:00").unwrap()
        );
        assert_eq!(
            ts[3].0,
            DateTime::parse_from_rfc3339("2021-04-15T06:27:31+00:00").unwrap()
        );
        assert_eq!(
            ts[4].0,
            DateTime::parse_from_rfc3339("2021-04-15T06:29:31+00:00").unwrap()
        );
        assert_eq!(ts[0].1, 0);
        assert_eq!(ts[1].1, 1);
        assert_eq!(ts[2].1, 0);
        assert_eq!(ts[3].1, 1);
        assert_eq!(ts[4].1, 0);
    }

    #[test]
    fn split_time_no_matches() {
        let reader = SplitTimeReader::default();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[2021-04-15T06:25:31+00:00] foo").unwrap();
        writeln!(file, "[2021-04-15T06:26:31+00:00] bar").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn split_time_zero_matches() {
        let mut builder = SplitTimeReaderBuilder::default();
        builder.matches(vec![
            "foo".to_string(),
            "bar".to_string(),
            "gnat".to_string(),
        ]);
        builder.ts_format(String::from("%Y_%m_%d %H:%M"));
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "_2021_04_15 06:25] none").unwrap();
        writeln!(file, "_2021_04_15 06:26] none").unwrap();
        writeln!(file, "[2021-04-15T06:25:31+00:00] foo").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn split_time_bad_guess() {
        let mut builder = SplitTimeReaderBuilder::default();
        builder.matches(vec![
            "foo".to_string(),
            "bar".to_string(),
            "gnat".to_string(),
        ]);
        let reader = builder.build().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "XXX none").unwrap();
        writeln!(file, "[2021-04-15T06:25:31+00:00] foo").unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn split_time_bad_file() {
        let reader = SplitTimeReader::default();
        let file = NamedTempFile::new().unwrap();
        let ts = reader.read(file.path().to_str().unwrap());
        assert_eq!(ts.len(), 0);
    }
}
