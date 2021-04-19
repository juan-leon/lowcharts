use std::env;

use clap::{AppSettings, Clap};
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

/// Tool to draw low-resolution graphs in terminal
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Input file.  If not present or a single dash, standard input will be used.
    #[clap(default_value = "-")]
    input: String,
    /// Filter out values bigger than this
    #[clap(long)]
    max: Option<f64>,
    /// Filter out values smaller than this
    #[clap(long)]
    min: Option<f64>,
    /// Use colors in the output.  Auto means "yes if tty with TERM != dumb and
    /// no redirects".
    #[clap(short, long, default_value = "auto", possible_values = &["auto", "no", "yes"])]
    color: String,
    /// Use this many characters as terminal width
    #[clap(long, default_value = "110")]
    width: usize,
    /// Use a regex to capture input values.  By default this will use a capture
    /// group named "value".  If not present, it will use first capture group.
    /// If not present, a number per line is expected.  Examples of regex are '
    /// 200 \d+ ([0-9.]+)' (1 anonymous capture group) or 'a(a)?
    /// (?P<value>[0-9.]+)' (a named capture group).
    #[clap(long)]
    regex: Option<String>,
    #[clap(long)]
    /// Be more verbose
    verbose: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Plot an histogram from input values
    Hist(Hist),
    /// Plot an 2d plot where y-values are averages of input values (as many
    /// averages as wide is the plot)
    Plot(Plot),
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Hist {
    /// Use that many intervals
    #[clap(long, default_value = "20")]
    intervals: usize,
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Plot {
    /// Use that many rows for the plot
    #[clap(long, default_value = "40")]
    height: usize,
}

fn main() {
    let opts: Opts = Opts::parse();
    disable_color_if_needed(&opts.color);
    let mut builder = reader::DataReaderBuilder::default();
    builder.verbose(opts.verbose);
    if opts.min.is_some() || opts.max.is_some() {
        builder.range(opts.min.unwrap_or(f64::NEG_INFINITY)..opts.max.unwrap_or(f64::INFINITY));
    }
    if let Some(string) = opts.regex {
        match Regex::new(&string) {
            Ok(re) => {
                builder.regex(re);
            }
            _ => eprintln!("[{}]: Failed to parse regex {}", Red.paint("ERROR"), string),
        };
    }
    let reader = builder.build().unwrap();

    let vec = reader.read(opts.input);
    if vec.is_empty() {
        eprintln!("[{}]: No data", Yellow.paint("WARN"));
        std::process::exit(0);
    }

    let stats = stats::Stats::new(&vec);
    match opts.subcmd {
        SubCommand::Hist(o) => {
            let mut histogram = histogram::Histogram::new(
                o.intervals,
                (stats.max - stats.min) / o.intervals as f64,
                stats,
            );
            histogram.load(&vec);
            println!("{:width$}", histogram, width = opts.width);
        }
        SubCommand::Plot(o) => {
            let mut plot = plot::Plot::new(opts.width, o.height, stats);
            plot.load(&vec);
            print!("{}", plot);
        }
    }
}
