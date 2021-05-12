mod app;
mod plot;
mod read;
mod stats;

use std::env;

#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate log;
use clap::ArgMatches;
use isatty::stdout_isatty;
use regex::Regex;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use yansi::Paint;

/// Sets up color choices and verbosity in the two libraries used for output:
/// simplelog and yansi
fn configure_output(option: &str, verbose: bool) {
    let mut color_choice = ColorChoice::Auto;
    match option {
        "no" => {
            Paint::disable();
            color_choice = ColorChoice::Never;
        }
        "auto" => match env::var("TERM") {
            Ok(value) if value == "dumb" => Paint::disable(),
            _ => {
                if !stdout_isatty() {
                    Paint::disable();
                }
            }
        },
        "yes" => {
            color_choice = ColorChoice::Always;
        }
        _ => (),
    };
    if let Err(err) = TermLogger::init(
        if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Trace)
            .set_thread_level(LevelFilter::Trace)
            .set_target_level(LevelFilter::Trace)
            .build(),
        TerminalMode::Stderr,
        color_choice,
    ) {
        // We trigger this error when unit testing this fn
        eprintln!("Error: {}", err);
    }
}

/// Build a reader able to read floats (potentially capturing them with regex)
/// from an input source.
fn get_float_reader(matches: &ArgMatches) -> read::DataReader {
    let mut builder = read::DataReaderBuilder::default();
    if matches.is_present("min") || matches.is_present("max") {
        let min = matches.value_of_t("min").unwrap_or(f64::NEG_INFINITY);
        let max = matches.value_of_t("max").unwrap_or(f64::INFINITY);
        if min > max {
            error!("Minimum should be smaller than maximum");
            std::process::exit(1);
        }
        builder.range(min..max);
    }
    if let Some(string) = matches.value_of("regex") {
        match Regex::new(&string) {
            Ok(re) => {
                builder.regex(re);
            }
            _ => {
                error!("Failed to parse regex {}", string);
                std::process::exit(1);
            }
        };
    }
    builder.build().unwrap()
}

/// Implements the hist cli-subcommand
fn histogram(matches: &ArgMatches) {
    let reader = get_float_reader(&matches);
    let vec = reader.read(matches.value_of("input").unwrap());
    if vec.is_empty() {
        warn!("No data to process");
        std::process::exit(0);
    }
    let stats = stats::Stats::new(&vec);
    let width = matches.value_of_t("width").unwrap();
    let mut intervals: usize = matches.value_of_t("intervals").unwrap();

    intervals = intervals.min(vec.len());
    let mut histogram =
        plot::Histogram::new(intervals, (stats.max - stats.min) / intervals as f64, stats);
    histogram.load(&vec);
    print!("{:width$}", histogram, width = width);
}

/// Implements the plot cli-subcommand
fn plot(matches: &ArgMatches) {
    let reader = get_float_reader(&matches);
    let vec = reader.read(matches.value_of("input").unwrap());
    if vec.is_empty() {
        warn!("No data to process");
        std::process::exit(0);
    }
    let mut plot = plot::XyPlot::new(
        matches.value_of_t("width").unwrap(),
        matches.value_of_t("height").unwrap(),
        stats::Stats::new(&vec),
    );
    plot.load(&vec);
    print!("{}", plot);
}

/// Implements the matches cli-subcommand
fn matchbar(matches: &ArgMatches) {
    let reader = read::DataReader::default();
    let width = matches.value_of_t("width").unwrap();
    print!(
        "{:width$}",
        reader.read_matches(
            matches.value_of("input").unwrap(),
            matches.values_of("match").unwrap().collect()
        ),
        width = width
    );
}

/// Implements the timehist cli-subcommand
fn timehist(matches: &ArgMatches) {
    let mut builder = read::TimeReaderBuilder::default();
    if let Some(string) = matches.value_of("regex") {
        match Regex::new(&string) {
            Ok(re) => {
                builder.regex(re);
            }
            _ => {
                error!("Failed to parse regex {}", string);
                std::process::exit(1);
            }
        };
    }
    if let Some(as_str) = matches.value_of("format") {
        builder.ts_format(as_str.to_string());
    }
    let width = matches.value_of_t("width").unwrap();
    let reader = builder.build().unwrap();
    let vec = reader.read(matches.value_of("input").unwrap());
    if vec.len() <= 1 {
        warn!("Not enough data to process");
        std::process::exit(0);
    }
    let mut timehist = plot::TimeHistogram::new(matches.value_of_t("intervals").unwrap(), &vec);
    timehist.load(&vec);

    print!("{:width$}", timehist, width = width);
}

fn main() {
    let matches = app::get_app().get_matches();
    configure_output(
        matches.value_of("color").unwrap(),
        matches.is_present("verbose"),
    );
    match matches.subcommand() {
        Some(("hist", subcommand_matches)) => {
            histogram(subcommand_matches);
        }
        Some(("plot", subcommand_matches)) => {
            plot(subcommand_matches);
        }
        Some(("matches", subcommand_matches)) => {
            matchbar(subcommand_matches);
        }
        Some(("timehist", subcommand_matches)) => {
            timehist(subcommand_matches);
        }
        _ => unreachable!("Invalid subcommand"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use yansi::Color::Blue;

    #[test]
    fn test_output_yes() {
        Paint::enable();
        configure_output("yes", true);
        let display = format!("{}", Blue.paint("blue"));
        assert_eq!("\u{1b}[34mblue\u{1b}[0m", display);
        assert_eq!(LevelFilter::Debug, log::max_level());
    }

    #[test]
    fn test_output_no() {
        Paint::enable();
        configure_output("no", false);
        let display = format!("{}", Blue.paint("blue"));
        assert_eq!("blue", display);
        assert_eq!(LevelFilter::Info, log::max_level());
    }

    #[test]
    fn test_output_auto() {
        Paint::enable();
        env::set_var("TERM", "dumb");
        configure_output("auto", false);
        let display = format!("{}", Blue.paint("blue"));
        assert_eq!("blue", display);
    }
}
