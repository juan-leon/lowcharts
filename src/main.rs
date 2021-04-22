use clap::ArgMatches;
use std::env;

use isatty::stdout_isatty;
use regex::Regex;
use yansi::Color::{Red, Yellow};
use yansi::Paint;

#[macro_use]
extern crate derive_builder;

mod app;
mod histogram;
mod matchbar;
mod plot;
mod reader;
mod stats;

fn disable_color_if_needed(option: &str) {
    match option {
        "no" => Paint::disable(),
        "auto" => match env::var("TERM") {
            Ok(value) if value == "dumb" => Paint::disable(),
            _ => {
                if !stdout_isatty() {
                    Paint::disable();
                }
            }
        },
        _ => (),
    }
}

fn get_reader(matches: &ArgMatches, verbose: bool) -> reader::DataReader {
    let mut builder = reader::DataReaderBuilder::default();
    builder.verbose(verbose);
    if matches.is_present("min") || matches.is_present("max") {
        let min = matches.value_of_t("min").unwrap_or(f64::NEG_INFINITY);
        let max = matches.value_of_t("max").unwrap_or(f64::INFINITY);
        if min > max {
            eprintln!(
                "[{}] Minimum should be smaller than maximum",
                Red.paint("ERROR")
            );
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
                eprintln!("[{}]: Failed to parse regex {}", Red.paint("ERROR"), string);
                std::process::exit(1);
            }
        };
    }
    builder.build().unwrap()
}

fn histogram(matches: &ArgMatches, verbose: bool) {
    let reader = get_reader(&matches, verbose);
    let vec = reader.read(matches.value_of("input").unwrap());
    if vec.is_empty() {
        eprintln!("[{}] No data to process", Yellow.paint("WARN"));
        std::process::exit(0);
    }
    let stats = stats::Stats::new(&vec);
    let width = matches.value_of_t("width").unwrap();
    let mut intervals: usize = matches.value_of_t("intervals").unwrap();

    intervals = intervals.min(vec.len());
    let mut histogram =
        histogram::Histogram::new(intervals, (stats.max - stats.min) / intervals as f64, stats);
    histogram.load(&vec);
    println!("{:width$}", histogram, width = width);
}

fn plot(matches: &ArgMatches, verbose: bool) {
    let reader = get_reader(&matches, verbose);
    let vec = reader.read(matches.value_of("input").unwrap());
    if vec.is_empty() {
        eprintln!("[{}] No data to process", Yellow.paint("WARN"));
        std::process::exit(0);
    }
    let mut plot = plot::Plot::new(
        matches.value_of_t("width").unwrap(),
        matches.value_of_t("height").unwrap(),
        stats::Stats::new(&vec),
    );
    plot.load(&vec);
    print!("{}", plot);
}

fn matchbar(matches: &ArgMatches) {
    let reader = reader::DataReader::default();
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

fn main() {
    let matches = app::get_app().get_matches();
    let verbose = matches.is_present("verbose");
    if let Some(c) = matches.value_of("color") {
        disable_color_if_needed(c);
    }
    match matches.subcommand() {
        Some(("hist", subcommand_matches)) => {
            histogram(subcommand_matches, verbose);
        }
        Some(("plot", subcommand_matches)) => {
            plot(subcommand_matches, verbose);
        }
        Some(("matches", subcommand_matches)) => {
            matchbar(subcommand_matches);
        }
        _ => unreachable!("Invalid subcommand"),
    };
}
