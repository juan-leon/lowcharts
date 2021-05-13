use clap::{self, App, AppSettings, Arg};

fn add_input(app: App) -> App {
    app.arg(
        Arg::new("input")
            .about("Input file")
            .default_value("-")
            .long_about("If not present or a single dash, standard input will be used"),
    )
}

fn add_min_max(app: App) -> App {
    app.arg(
        Arg::new("max")
            .long("max")
            .short('M')
            .about("Filter out values bigger than this")
            .takes_value(true),
    )
    .arg(
        Arg::new("min")
            .long("min")
            .short('m')
            .about("Filter out values smaller than this")
            .takes_value(true),
    )
}

fn add_regex(app: App) -> App {
    const LONG_RE_ABOUT: &str = "\
A regular expression used for capturing the values to be plotted inside input
lines.

By default this will use a capture group named `value`.  If not present, it will
use first capture group.

If no regex is used, a number per line is expected (something that can be parsed
as float).

Examples of regex are ' 200 \\d+ ([0-9.]+)' (where there is one anonymous capture
group) and 'a(a)? (?P<value>[0-9.]+)' (where there are two capture groups, and
the named one will be used).
";
    app.arg(
        Arg::new("regex")
            .long("regex")
            .short('R')
            .about("Use a regex to capture input values")
            .long_about(LONG_RE_ABOUT)
            .takes_value(true),
    )
}

fn add_non_capturing_regex(app: App) -> App {
    app.arg(
        Arg::new("regex")
            .long("regex")
            .short('R')
            .about("Filter out lines where regex is notr present")
            .takes_value(true),
    )
}

fn add_width(app: App) -> App {
    app.arg(
        Arg::new("width")
            .long("width")
            .short('w')
            .about("Use this many characters as terminal width")
            .default_value("110")
            .takes_value(true),
    )
}

fn add_intervals(app: App) -> App {
    app.arg(
        Arg::new("intervals")
            .long("intervals")
            .short('i')
            .about("Use no more than this amount of buckets to classify data")
            .default_value("20")
            .takes_value(true),
    )
}

pub fn get_app() -> App<'static> {
    let mut hist = App::new("hist")
        .version(clap::crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("Plot an histogram from input values");
    hist = add_input(add_regex(add_width(add_min_max(add_intervals(hist)))));

    let mut plot = App::new("plot")
        .version(clap::crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("Plot an 2d x-y graph where y-values are averages of input values")
        .arg(
            Arg::new("height")
                .long("height")
                .short('h')
                .about("Use that many `rows` for the plot")
                .default_value("40")
                .takes_value(true),
        );
    plot = add_input(add_regex(add_width(add_min_max(plot))));

    let mut matches = App::new("matches")
        .version(clap::crate_version!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::AllowMissingPositional)
        .about("Plot barchar with counts of occurences of matches params");
    matches = add_input(add_width(matches)).arg(
        Arg::new("match")
            .about("Count maches for those strings")
            .required(true)
            .takes_value(true)
            .multiple(true),
    );

    let mut timehist = App::new("timehist")
        .version(clap::crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("Plot histogram with amount of matches over time")
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .about("Use this string formatting")
                .takes_value(true),
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .about("Cap the time interval at that duration (example: '3h 5min')")
                .takes_value(true),
        )
        .arg(Arg::new("early-stop").long("early-stop").about(
            "If duration flag is used, assume monotonic times and stop as soon as possible",
        ));
    timehist = add_input(add_width(add_non_capturing_regex(add_intervals(timehist))));

    App::new("lowcharts")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .max_term_width(100)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .about("Use colors in the output")
                .possible_values(&["auto", "no", "yes"])
                .default_value("auto")
                .takes_value(true),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("Be more verbose")
                .takes_value(false),
        )
        .subcommand(hist)
        .subcommand(plot)
        .subcommand(matches)
        .subcommand(timehist)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn hist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "--verbose", "hist", "foo"];
        let m = get_app().get_matches_from(arg_vec);
        assert!(m.is_present("verbose"));
        if let Some(sub_m) = m.subcommand_matches("hist") {
            assert_eq!("foo", sub_m.value_of("input").unwrap());
            assert!(sub_m.value_of("max").is_none());
            assert!(sub_m.value_of("min").is_none());
            assert!(sub_m.value_of("regex").is_none());
            assert_eq!("110", sub_m.value_of("width").unwrap());
            assert_eq!("20", sub_m.value_of("intervals").unwrap());
        } else {
            assert!(false, "Subcommand `hist` not detected");
        }
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
        if let Some(sub_m) = m.subcommand_matches("plot") {
            assert_eq!("-", sub_m.value_of("input").unwrap());
            assert_eq!("1.1", sub_m.value_of("max").unwrap());
            assert_eq!("0.9", sub_m.value_of("min").unwrap());
            assert_eq!("11", sub_m.value_of("height").unwrap());
        } else {
            assert!(false, "Subcommand `plot` not detected");
        }
    }

    #[test]
    fn matches_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "matches", "-", "A", "B", "C"];
        let m = get_app().get_matches_from(arg_vec);
        if let Some(sub_m) = m.subcommand_matches("matches") {
            assert_eq!("-", sub_m.value_of("input").unwrap());
            assert_eq!(
                // vec![String::from("A"), String::from("B"), String::from("C")],
                vec!["A", "B", "C"],
                sub_m.values_of("match").unwrap().collect::<Vec<&str>>()
            );
        } else {
            assert!(false, "Subcommand `matches` not detected");
        }
    }

    #[test]
    fn timehist_subcommand_arg_parsing() {
        let arg_vec = vec!["lowcharts", "timehist", "--regex", "foo", "some"];
        let m = get_app().get_matches_from(arg_vec);
        if let Some(sub_m) = m.subcommand_matches("timehist") {
            assert_eq!("some", sub_m.value_of("input").unwrap());
            assert_eq!("foo", sub_m.value_of("regex").unwrap());
        } else {
            assert!(false, "Subcommand `timehist` not detected");
        }
    }
}
