use clap::ArgMatches;
use std::env;

use isatty::stdout_isatty;
use regex::Regex;
use yansi::Color::Red;
use yansi::Color::Yellow;
use yansi::Paint;

#[macro_use]
extern crate derive_builder;

mod histogram;
mod plot;
mod reader;
mod stats;
mod app;

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
            eprintln!("[{}] Minimum should be smaller than maximum", Red.paint("ERROR"));
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


fn main() {
    let matches = app::get_app().get_matches();

    if let Some(c) = matches.value_of("color") {
        disable_color_if_needed(c);
    }

    let sub_matches = match matches.subcommand_name() {
        Some("hist") => {
            matches.subcommand_matches("hist").unwrap()
        },
        Some("plot") => {
            matches.subcommand_matches("plot").unwrap()
        },
        _ => {
            eprintln!("[{}] Invalid subcommand", Red.paint("ERROR"));
            std::process::exit(1);
        }
    };
    let reader = get_reader(&sub_matches, matches.is_present("verbose"));

    let vec = reader.read(sub_matches.value_of("input").unwrap_or("-"));
    if vec.is_empty() {
        eprintln!("[{}]: No data to process", Yellow.paint("WARN"));
        std::process::exit(0);
    }
    let stats = stats::Stats::new(&vec);
    let width = sub_matches.value_of_t("width").unwrap();
    match matches.subcommand_name() {
        Some("hist") => {
            let mut intervals: usize = sub_matches.value_of_t("intervals").unwrap();
            intervals = intervals.min(vec.len());
            let mut histogram = histogram::Histogram::new(
                intervals,
                (stats.max - stats.min) / intervals as f64,
                stats,
            );
            histogram.load(&vec);
            println!("{:width$}", histogram, width = width);
        },
        Some("plot") => {
            let mut plot = plot::Plot::new(width, sub_matches.value_of_t("height").unwrap(), stats);
            plot.load(&vec);
            print!("{}", plot);
        },
        _ => ()
    };
}
