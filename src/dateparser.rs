use std::ops::Range;

use chrono::{DateTime, FixedOffset, Utc, NaiveDateTime, NaiveTime, ParseError, TimeZone};
use regex::Regex;

type DateParsingFun = fn(s: &str) -> Result<DateTime<FixedOffset>, ParseError>;

// Those are some date formats that are common for my personal (and biased)
// experience.  So, there is logic to detect and parse them.
const PARSE_SPECIFIERS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S,%3f", // python %(asctime)s
    "%Y-%m-%d %H:%M:%S",
    "%Y/%m/%d %H:%M:%S",  // Seen in some nginx logs
    "%d-%b-%Y::%H:%M:%S", // Seen in rabbitmq logs
    "%H:%M:%S",           // strace -t
    "%H:%M:%S.%6f",       // strace -tt (-ttt generates timestamps)
];

// Max length that a timestamp can have
const MAX_LEN: usize = 28;

pub struct LogDateParser<'a> {
    range: Range<usize>,
    parser: Option<DateParsingFun>,
    ts_format: Option<&'a str>,
}

impl<'a> LogDateParser<'a> {
    pub fn new_with_guess(log_line: &str) -> Result<LogDateParser<'_>, String> {
        if let Some(x) = Self::from_brackets(log_line) {
            Ok(x)
        } else if let Some(x) = Self::from_heuristic(log_line) {
            Ok(x)
        } else {
            Err(format!("Could not parse a timestamp in {}", log_line))
        }
    }

    pub fn new_with_format(
        log_line: &str,
        format_string: &'a str,
    ) -> Result<LogDateParser<'a>, String> {
        // We look for where the timestamp is in logs using a brute force
        // approach with 1st log line, but capping the max length we scan for
        for i in 0..log_line.len() {
            for j in (i..(i + (MAX_LEN * 2)).min(log_line.len() + 1)).rev() {
                if NaiveDateTime::parse_from_str(&log_line[i..j], format_string).is_ok() {
                    // I would like to capture ts_format in a closure and assign
                    // it to parser, but I cannot coerce a capturing closure to
                    // a typed fn.  I still need to learn the idiomatic way of
                    // dealing with this.
                    return Ok(LogDateParser {
                        range: i..j,
                        parser: None,
                        ts_format: Some(format_string),
                    });
                }
            }
        }
        Err(format!(
            "Could locate a '{}' timestamp in '{}'",
            format_string, log_line
        ))
    }

    pub fn parse(&self, s: &str) -> Result<DateTime<FixedOffset>, ParseError> {
        let range = self.range.start.min(s.len())..self.range.end.min(s.len());
        match self.parser {
            Some(p) => p(&s[range]),
            None => match NaiveDateTime::parse_from_str(&s[range], self.ts_format.unwrap()) {
                Ok(naive) => {
                    let date_time: DateTime<Utc> = Utc.from_local_datetime(&naive).unwrap();
                    Ok(date_time.with_timezone(&TimeZone::from_offset(&FixedOffset::west(0))))
                }
                Err(err) => Err(err),
            },
        }
    }

    fn guess_parser(s: &str) -> Option<DateParsingFun> {
        if DateTime::parse_from_rfc3339(s).is_ok() {
            Some(DateTime::parse_from_rfc3339)
        } else if DateTime::parse_from_rfc2822(s).is_ok() {
            Some(DateTime::parse_from_rfc2822)
        } else if Self::looks_like_timestamp(&s) {
            Some(|string: &str| {
                let dot = match string.find('.') {
                    Some(x) => x,
                    None => string.len(),
                };
                let nanosecs = if dot < string.len() {
                    let missing_zeros = (10 + dot - string.len()) as u32;
                    match string[dot + 1..].parse::<u32>() {
                        Ok(x) => x * 10_u32.pow(missing_zeros),
                        _ => 0,
                    }
                } else {
                    0
                };
                match string[..dot].parse::<i64>() {
                    Ok(secs) => {
                        let naive = NaiveDateTime::from_timestamp(secs, nanosecs);
                        let date_time: DateTime<Utc> = Utc.from_local_datetime(&naive).unwrap();
                        Ok(date_time.with_timezone(&TimeZone::from_offset(&FixedOffset::west(0))))
                    }
                    Err(_) => DateTime::parse_from_rfc3339(""),
                }
            })
        } else if NaiveDateTime::parse_from_str(s, PARSE_SPECIFIERS[0]).is_ok() {
            // TODO: All of this stuff below should be rewritten using macros.
            // Reason for "repeating myself" is that I cannot coerce closures to
            // fn types if they capture variables (an index to PARSE_SPECIFIERS,
            // for instance).
            Some(
                |string: &str| match NaiveDateTime::parse_from_str(string, PARSE_SPECIFIERS[0]) {
                    Ok(naive) => {
                        let date_time: DateTime<Utc> = Utc.from_local_datetime(&naive).unwrap();
                        Ok(date_time.with_timezone(&TimeZone::from_offset(&FixedOffset::west(0))))
                    }
                    Err(err) => Err(err),
                },
            )
        } else if NaiveDateTime::parse_from_str(s, PARSE_SPECIFIERS[1]).is_ok() {
            Some(
                |string: &str| match NaiveDateTime::parse_from_str(string, PARSE_SPECIFIERS[1]) {
                    Ok(naive) => {
                        let date_time: DateTime<Utc> = Utc.from_local_datetime(&naive).unwrap();
                        Ok(date_time.with_timezone(&TimeZone::from_offset(&FixedOffset::west(0))))
                    }
                    Err(err) => Err(err),
                },
            )
        } else if NaiveDateTime::parse_from_str(s, PARSE_SPECIFIERS[2]).is_ok() {
            Some(
                |string: &str| match NaiveDateTime::parse_from_str(string, PARSE_SPECIFIERS[2]) {
                    Ok(naive) => {
                        let date_time: DateTime<Utc> = Utc.from_local_datetime(&naive).unwrap();
                        Ok(date_time.with_timezone(&TimeZone::from_offset(&FixedOffset::west(0))))
                    }
                    Err(err) => Err(err),
                },
            )
        } else if NaiveDateTime::parse_from_str(s, PARSE_SPECIFIERS[3]).is_ok() {
            Some(
                |string: &str| match NaiveDateTime::parse_from_str(string, PARSE_SPECIFIERS[3]) {
                    Ok(naive) => {
                        let date_time: DateTime<Utc> = Utc.from_local_datetime(&naive).unwrap();
                        Ok(date_time.with_timezone(&TimeZone::from_offset(&FixedOffset::west(0))))
                    }
                    Err(err) => Err(err),
                },
            )
        } else if NaiveTime::parse_from_str(s, PARSE_SPECIFIERS[4]).is_ok() {
            Some(
                |string: &str| match NaiveTime::parse_from_str(string, PARSE_SPECIFIERS[4]) {
                    Ok(naive_time) => Ok(Utc::today()
                        .and_time(naive_time)
                        .unwrap()
                        .with_timezone(&TimeZone::from_offset(&FixedOffset::west(0)))),
                    Err(err) => Err(err),
                },
            )
        } else if NaiveTime::parse_from_str(s, PARSE_SPECIFIERS[5]).is_ok() {
            Some(
                |string: &str| match NaiveTime::parse_from_str(string, PARSE_SPECIFIERS[5]) {
                    Ok(naive_time) => Ok(Utc::today()
                        .and_time(naive_time)
                        .unwrap()
                        .with_timezone(&TimeZone::from_offset(&FixedOffset::west(0)))),
                    Err(err) => Err(err),
                },
            )
        } else {
            None
        }
    }

    fn from_brackets(s: &str) -> Option<LogDateParser> {
        match s.chars().next() {
            Some('[') => {
                if let Some(x) = s.find(']') {
                    match Self::guess_parser(&s[1..x]) {
                        Some(parser) => Some(LogDateParser {
                            range: 1..x,
                            parser: Some(parser),
                            ts_format: None,
                        }),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn from_heuristic(s: &str) -> Option<LogDateParser> {
        // First we locate the first digit
        for (i, c) in s.chars().enumerate() {
            if c.is_digit(10) {
                for j in (i..(i + MAX_LEN).min(s.len() + 1)).rev() {
                    if let Some(parser) = Self::guess_parser(&s[i..j]) {
                        return Some(LogDateParser {
                            range: i..j,
                            parser: Some(parser),
                            ts_format: None,
                        });
                    }
                }
                break;
            }
        }
        None
    }

    // Returns true if string looks like a unix-like timestamp of arbitrary
    // precision
    fn looks_like_timestamp(s: &str) -> bool {
        Regex::new(r"^[0-9]{10}(\.[0-9]{1,9})?$")
            .unwrap()
            .is_match(s)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_rfc3339_brackets() {
        let r = LogDateParser::new_with_guess("[1996-12-19T16:39:57-08:00] foobar").unwrap();
        assert_eq!(
            r.parse("[2096-11-19T16:39:57-08:00]"),
            DateTime::parse_from_rfc3339("2096-11-19T16:39:57-08:00")
        );
    }

    #[test]
    fn test_rfc3339_no_brackets() {
        let r = LogDateParser::new_with_guess("2021-04-25T16:57:15.337Z foobar").unwrap();
        assert_eq!(
            r.parse("2031-04-25T16:57:15.337Z"),
            DateTime::parse_from_rfc3339("2031-04-25T16:57:15.337Z")
        );
    }

    #[test]
    fn test_rfc2822() {
        let r = LogDateParser::new_with_guess("12 Jul 2003 10:52:37 +0200 foobar").unwrap();
        assert_eq!(
            r.parse("22 Jun 2003 10:52:37 +0500"),
            DateTime::parse_from_rfc2822("22 Jun 2003 10:52:37 +0500")
        );
    }

    #[test]
    fn test_bad_bracket() {
        let r = LogDateParser::new_with_guess("[12 Jul 2003 10:52:37 +0200 foobar").unwrap();
        assert_eq!(
            r.parse("[22 Jun 2003 10:52:37 +0500"),
            DateTime::parse_from_rfc2822("22 Jun 2003 10:52:37 +0500")
        );
    }

    #[test]
    fn test_prefix() {
        let r = LogDateParser::new_with_guess("foobar 1996-12-19T16:39:57-08:00 foobar").unwrap();
        assert_eq!(
            r.parse("foobar 2096-11-19T16:39:57-08:00"),
            DateTime::parse_from_rfc3339("2096-11-19T16:39:57-08:00")
        );
    }

    #[test]
    fn test_bad_format() {
        assert!(LogDateParser::new_with_guess("996-12-19T16:39:57-08:00 foobar").is_err());
    }

    #[test]
    fn test_short_line() {
        assert!(LogDateParser::new_with_guess("9").is_err());
    }

    #[test]
    fn test_empty_line() {
        assert!(LogDateParser::new_with_guess("").is_err());
    }

    #[test]
    fn test_timestamps() {
        let r = LogDateParser::new_with_guess("ts 1619688527.018165").unwrap();
        assert_eq!(
            r.parse("ts 1619655527.888165"),
            DateTime::parse_from_rfc3339("2021-04-29T00:18:47.888165+00:00")
        );
        let r = LogDateParser::new_with_guess("1619688527.123").unwrap();
        assert_eq!(
            r.parse("1619655527.123"),
            DateTime::parse_from_rfc3339("2021-04-29T00:18:47.123+00:00")
        );
    }

    #[test]
    fn test_known_formats() {
        let r = LogDateParser::new_with_guess("2021-04-28 06:25:24,321").unwrap();
        assert_eq!(
            r.parse("2021-04-28 06:25:24,321"),
            DateTime::parse_from_rfc3339("2021-04-28T06:25:24.321+00:00")
        );
        let r = LogDateParser::new_with_guess("2021-04-28 06:25:24").unwrap();
        assert_eq!(
            r.parse("2021-04-28 06:25:24"),
            DateTime::parse_from_rfc3339("2021-04-28T06:25:24+00:00")
        );
        let r = LogDateParser::new_with_guess("28-Apr-2021::12:10:42").unwrap();
        assert_eq!(
            r.parse("28-Apr-2021::12:10:42"),
            DateTime::parse_from_rfc3339("2021-04-28T12:10:42+00:00")
        );
        let r = LogDateParser::new_with_guess("2019/12/19 05:01:02").unwrap();
        assert_eq!(
            r.parse("2019/12/19 05:01:02"),
            DateTime::parse_from_rfc3339("2019-12-19T05:01:02+00:00")
        );
        let r = LogDateParser::new_with_guess("11:29:13.120535").unwrap();
        let now_as_date = format!("{}", Utc::today());
        assert_eq!(
            r.parse("11:29:13.120535"),
            DateTime::parse_from_rfc3339(&format!(
                "{}{}",
                &now_as_date[..10],
                "T11:29:13.120535+00:00"
            ))
        );
        let r = LogDateParser::new_with_guess("11:29:13").unwrap();
        assert_eq!(
            r.parse("11:29:13.120535"),
            DateTime::parse_from_rfc3339(&format!("{}{}", &now_as_date[..10], "T11:29:13+00:00"))
        );
    }

    #[test]
    fn test_tricky_line() {
        let r = LogDateParser::new_with_guess("[1996-12-19T16:39:57-08:00] foobar").unwrap();
        assert!(r.parse("nothing").is_err());
    }

    #[test]
    fn test_custom_format() {
        assert!(LogDateParser::new_with_format(
            "[1996-12-19T16:39:57-08:00] foobar",
            "%Y-%m-%d %H:%M:%S"
        )
        .is_err());
        let r = LogDateParser::new_with_format("[1996-12-19 16-39-57] foobar", "%Y-%m-%d %H-%M-%S")
            .unwrap();
        assert_eq!(
            r.parse("[2096-11-19 04-25-24]"),
            DateTime::parse_from_rfc3339("2096-11-19T04:25:24+00:00")
        );
    }
}
