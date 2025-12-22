use clap::{
    self,
    builder::styling::{AnsiColor, Effects, Styles},
    value_parser, Arg, ArgAction, Command,
};

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
            .value_parser(value_parser!(String)),
    )
}

fn add_min_max(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("max")
            .long("max")
            .short('M')
            .allow_hyphen_values(true)
            .help("Filter out values bigger than this")
            .value_parser(value_parser!(f64)),
    )
    .arg(
        Arg::new("min")
            .long("min")
            .short('m')
            .allow_hyphen_values(true)
            .help("Filter out values smaller than this")
            .value_parser(value_parser!(f64)),
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
            .value_parser(value_parser!(String)),
    )
}

fn add_non_capturing_regex(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("regex")
            .long("regex")
            .short('R')
            .help("Filter out lines where regex is not present")
            .value_parser(value_parser!(String)),
    )
}

fn add_width(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("width")
            .long("width")
            .short('w')
            .help("Use this many characters as terminal width")
            .default_value("110")
            .value_parser(value_parser!(usize)),
    )
}

fn add_intervals(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("intervals")
            .long("intervals")
            .short('i')
            .help("Use no more than this amount of buckets to classify data")
            .default_value("20")
            .value_parser(value_parser!(usize)),
    )
}

fn add_precision(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("precision")
            .long("precision")
            .short('p')
            .help("Show that number of decimals (if omitted, 'human' units will be used)")
            .default_value("-1")
            .value_parser(value_parser!(i32)),
    )
}

fn add_log_scale(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("log-scale")
            .long("log-scale")
            .help("Use a logarithmic scale in buckets")
            .action(ArgAction::SetTrue),
    )
}

pub fn get_app() -> Command {
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
                .value_parser(value_parser!(usize)),
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
            .action(ArgAction::Append)
            .num_args(1..),
    );

    let mut timehist = Command::new("timehist")
        .version(clap::crate_version!())
        .about("Plot histogram with amount of matches over time")
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Use this string formatting")
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .help("Cap the time interval at that duration (example: '3h 5min')")
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("early-stop")
                .long("early-stop")
                .help(
                    "If duration flag is used, assume monotonic times and stop as soon as possible",
                )
                .action(ArgAction::SetTrue),
        );
    timehist = add_input(add_width(add_non_capturing_regex(add_intervals(timehist))));

    let mut splittimehist = Command::new("split-timehist")
        .version(clap::crate_version!())
        .about("Plot histogram of with amount of matches over time, split per match type")
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Use this string formatting")
                .value_parser(value_parser!(String)),
        );
    splittimehist = add_input_as_option(add_width(add_intervals(splittimehist))).arg(
        Arg::new("match")
            .help("Count matches for those strings")
            .required(true)
            .action(ArgAction::Append)
            .num_args(1..),
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
            .value_parser(value_parser!(i32)),
    );

    Command::new("lowcharts")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .styles(
            Styles::styled()
                .header(AnsiColor::Red.on_default() | Effects::BOLD)
                .usage(AnsiColor::Red.on_default() | Effects::BOLD)
                .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
                .placeholder(AnsiColor::Green.on_default()),
        )
        .max_term_width(100)
        .subcommand_required(true)
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .help("Use colors in the output")
                .value_parser(["auto", "no", "yes"])
                .default_value("auto")
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Be more verbose")
                .action(ArgAction::SetTrue),
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
        assert!(m.get_flag("verbose"));
        let sub_m = m.subcommand_matches("hist").unwrap();
        assert_eq!("foo", sub_m.get_one::<String>("input").unwrap());
        assert!(sub_m.get_one::<f64>("max").is_none());
        assert!(sub_m.get_one::<f64>("min").is_none());
        assert!(sub_m.get_one::<String>("regex").is_none());
        assert_eq!(&110usize, sub_m.get_one::<usize>("width").unwrap());
        assert_eq!(&20usize, sub_m.get_one::<usize>("intervals").unwrap());
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
        assert!(!m.get_flag("verbose"));
        let sub_m = m.subcommand_matches("plot").unwrap();
        assert_eq!("-", sub_m.get_one::<String>("input").unwrap());
        assert_eq!(&1.1f64, sub_m.get_one::<f64>("max").unwrap());
        assert_eq!(&0.9f64, sub_m.get_one::<f64>("min").unwrap());
        assert_eq!(&11usize, sub_m.get_one::<usize>("height").unwrap());
    }

    #[test]
    fn matches_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "matches", "A", "B", "C"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("matches").unwrap();
        assert_eq!("-", sub_m.get_one::<String>("input").unwrap());
        assert_eq!(
            vec!["A", "B", "C"],
            sub_m
                .get_many::<String>("match")
                .unwrap()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
        );
        let arg_vec = vec!["lowcharts", "matches", "A", "--input", "B", "C"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("matches").unwrap();
        assert_eq!("B", sub_m.get_one::<String>("input").unwrap());
        assert_eq!(
            vec!["A", "C"],
            sub_m
                .get_many::<String>("match")
                .unwrap()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
        );
    }

    #[test]
    fn timehist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "timehist", "--regex", "foo", "some"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("timehist").unwrap();
        assert_eq!("some", sub_m.get_one::<String>("input").unwrap());
        assert_eq!("foo", sub_m.get_one::<String>("regex").unwrap());
    }

    #[test]
    fn splittimehist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "split-timehist", "foo", "bar"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("split-timehist").unwrap();
        assert_eq!(
            vec!["foo", "bar"],
            sub_m
                .get_many::<String>("match")
                .unwrap()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
        );
    }

    #[test]
    fn terms_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "common-terms", "--regex", "foo", "some"];
        let m = get_app().get_matches_from(arg_vec);
        let sub_m = m.subcommand_matches("common-terms").unwrap();
        assert_eq!("some", sub_m.get_one::<String>("input").unwrap());
        assert_eq!("foo", sub_m.get_one::<String>("regex").unwrap());
    }
}
