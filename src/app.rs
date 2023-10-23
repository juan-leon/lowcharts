use clap::{self, Arg, Command};

fn add_input(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("input")
            .help("Input file")
            .default_value("-")
            .long_help("If not present or a single dash, standard input will be used"),
    )
}

fn add_input_as_option(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("input")
            .long("input")
            .default_value("-")
            .long_help("If not present or a single dash, standard input will be used")
            .takes_value(true),
    )
}

fn add_min_max(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("max")
            .long("max")
            .short('M')
            .allow_hyphen_values(true)
            .help("Filter out values bigger than this")
            .takes_value(true),
    )
    .arg(
        Arg::new("min")
            .long("min")
            .short('m')
            .allow_hyphen_values(true)
            .help("Filter out values smaller than this")
            .takes_value(true),
    )
}

fn add_regex(cmd: Command) -> Command {
    const LONG_RE_ABOUT: &str = "\
A regular expression used for capturing the values to be plotted inside input
lines.

By default this will use a capture group named `value`.  If not present, it will
use first capture group.

If no regex is used, the whole input lines will be matched.

Examples of regex are ' 200 \\d+ ([0-9.]+)' (where there is one anonymous capture
group) and 'a(a)? (?P<value>[0-9.]+)' (where there are two capture groups, and
the named one will be used).
";
    cmd.arg(
        Arg::new("regex")
            .long("regex")
            .short('R')
            .help("Use a regex to capture input values")
            .long_help(LONG_RE_ABOUT)
            .takes_value(true),
    )
}

fn add_non_capturing_regex(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("regex")
            .long("regex")
            .short('R')
            .help("Filter out lines where regex is not present")
            .takes_value(true),
    )
}

fn add_width(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("width")
            .long("width")
            .short('w')
            .help("Use this many characters as terminal width")
            .default_value("110")
            .takes_value(true),
    )
}

fn add_intervals(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("intervals")
            .long("intervals")
            .short('i')
            .help("Use no more than this amount of buckets to classify data")
            .default_value("20")
            .takes_value(true),
    )
}

fn add_precision(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("precision")
            .long("precision")
            .short('p')
            .help("Show that number of decimals (if omitted, 'human' units will be used)")
            .default_value("-1")
            .takes_value(true),
    )
}

fn add_log_scale(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("log-scale")
            .long("log-scale")
            .help("Use a logarithmic scale in buckets")
            .takes_value(false),
    )
}

pub fn get_app() -> Command<'static> {
    let mut hist = Command::new("hist")
        .version(clap::crate_version!())
        .about("Plot an histogram from input values");
    hist = add_input(add_regex(add_width(add_min_max(add_precision(
        add_intervals(add_log_scale(hist)),
    )))));

    let mut plot = Command::new("plot")
        .version(clap::crate_version!())
        .about("Plot an 2d x-y graph where y-values are averages of input values")
        .arg(
            Arg::new("height")
                .long("height")
                .short('H')
                .help("Use that many `rows` for the plot")
                .default_value("40")
                .takes_value(true),
        );
    plot = add_input(add_regex(add_width(add_min_max(add_precision(plot)))));

    let mut matches = Command::new("matches")
        .version(clap::crate_version!())
        .allow_missing_positional(true)
        .about("Plot barchar with counts of occurrences of matches params");
    matches = add_input_as_option(add_width(matches)).arg(
        Arg::new("match")
            .help("Count matches for those strings")
            .required(true)
            .takes_value(true)
            .multiple_occurrences(true),
    );

    let mut timehist =
        Command::new("timehist")
            .version(clap::crate_version!())
            .about("Plot histogram with amount of matches over time")
            .arg(
                Arg::new("format")
                    .long("format")
                    .short('f')
                    .help("Use this string formatting")
                    .takes_value(true),
            )
            .arg(
                Arg::new("duration")
                    .long("duration")
                    .help("Cap the time interval at that duration (example: '3h 5min')")
                    .takes_value(true),
            )
            .arg(Arg::new("early-stop").long("early-stop").help(
                "If duration flag is used, assume monotonic times and stop as soon as possible",
            ));
    timehist = add_input(add_width(add_non_capturing_regex(add_intervals(timehist))));

    let mut splittimehist = Command::new("split-timehist")
        .version(clap::crate_version!())
        .about("Plot histogram of with amount of matches over time, split per match type")
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Use this string formatting")
                .takes_value(true),
        );
    splittimehist = add_input_as_option(add_width(add_intervals(splittimehist))).arg(
        Arg::new("match")
            .help("Count matches for those strings")
            .required(true)
            .takes_value(true)
            .multiple_occurrences(true),
    );

    let mut common_terms = Command::new("common-terms")
        .version(clap::crate_version!())
        .about("Plot histogram with most common terms in input lines");
    common_terms = add_input(add_regex(add_width(common_terms))).arg(
        Arg::new("lines")
            .long("lines")
            .short('l')
            .help("Display that many lines, sorting by most frequent")
            .default_value("10")
            .takes_value(true),
    );

    Command::new("lowcharts")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .max_term_width(100)
        .subcommand_required(true)
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .help("Use colors in the output")
                .possible_values(["auto", "no", "yes"])
                .default_value("auto")
                .takes_value(true),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Be more verbose")
                .takes_value(false),
        )
        .subcommand(hist)
        .subcommand(plot)
        .subcommand(matches)
        .subcommand(timehist)
        .subcommand(splittimehist)
        .subcommand(common_terms)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn hist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "--verbose", "hist", "foo"];
        let m = get_app().get_matches_from(arg_vec);
        assert!(m.is_present("verbose"));
        let sub_m = m.subcommand_matches("hist").unwrap();
        assert_eq!("foo", sub_m.value_of("input").unwrap());
        assert!(sub_m.value_of("max").is_none());
        assert!(sub_m.value_of("min").is_none());
        assert!(sub_m.value_of("regex").is_none());
        assert_eq!("110", sub_m.value_of("width").unwrap());
        assert_eq!("20", sub_m.value_of("intervals").unwrap());
    }

    #[test]
    fn plot_subcommand_arg_parsing() {
        let arg_vec = vec![
            "lowcharts",
            "plot",
            "--max",
            "1.1",
            "-m",
            "0.9",
            "--height",
            "11",
        ];
        let m = get_app().get_matches_from(arg_vec);
        assert!(!m.is_present("verbose"));
        let sub_m = m.subcommand_matches("plot").unwrap();
        assert_eq!("-", sub_m.value_of("input").unwrap());
        assert_eq!("1.1", sub_m.value_of("max").unwrap());
        assert_eq!("0.9", sub_m.value_of("min").unwrap());
        assert_eq!("11", sub_m.value_of("height").unwrap());
    }

    #[test]
    fn matches_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "matches", "A", "B", "C"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("matches").unwrap();
        assert_eq!("-", sub_m.value_of("input").unwrap());
        assert_eq!(
            vec!["A", "B", "C"],
            sub_m.values_of("match").unwrap().collect::<Vec<&str>>()
        );
        let arg_vec = vec!["lowcharts", "matches", "A", "--input", "B", "C"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("matches").unwrap();
        assert_eq!("B", sub_m.value_of("input").unwrap());
        assert_eq!(
            vec!["A", "C"],
            sub_m.values_of("match").unwrap().collect::<Vec<&str>>()
        );
    }

    #[test]
    fn timehist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "timehist", "--regex", "foo", "some"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("timehist").unwrap();
        assert_eq!("some", sub_m.value_of("input").unwrap());
        assert_eq!("foo", sub_m.value_of("regex").unwrap());
    }

    #[test]
    fn splittimehist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "split-timehist", "foo", "bar"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("split-timehist").unwrap();
        assert_eq!(
            vec!["foo", "bar"],
            sub_m.values_of("match").unwrap().collect::<Vec<&str>>()
        );
    }

    #[test]
    fn terms_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "common-terms", "--regex", "foo", "some"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("common-terms").unwrap();
        assert_eq!("some", sub_m.value_of("input").unwrap());
        assert_eq!("foo", sub_m.value_of("regex").unwrap());
    }
}
