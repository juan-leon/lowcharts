use clap::{self, App, Arg, AppSettings};

fn add_common_options (app: App) -> App {

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

    app
        .arg(
            Arg::new("max")
                .long("max")
                .short('M')
                .about("Filter out values bigger than this")
                .takes_value(true)
        )
        .arg(
            Arg::new("min")
                .long("min")
                .short('m')
                .about("Filter out values smaller than this")
                .takes_value(true)
        )
        .arg(
            Arg::new("width")
                .long("width")
                .short('w')
                .about("Use this many characters as terminal width")
                .default_value("110")
                .takes_value(true)
        )
        .arg(
            Arg::new("regex")
                .long("regex")
                .short('R')
                .about("Use a regex to capture input values")
                .long_about(LONG_RE_ABOUT)
                .takes_value(true)
        )
        .arg(
            Arg::new("input")
                .about("Input file")
                .default_value("-")
                .long_about("If not present or a single dash, standard input will be used")
        )

}

pub fn get_app() -> App<'static> {

    let mut hist = App::new("hist")
        .version(clap::crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("Plot an histogram from input values")
        .arg(
            Arg::new("intervals")
                .long("intervals")
                .short('i')
                .about("Use that many buckets to classify data")
                .default_value("20")
                .takes_value(true)
        );

    hist = add_common_options(hist);

    let mut plot = App::new("plot")
        .version(clap::crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("Plot an 2d plot where y-values are averages of input values")
        .arg(
            Arg::new("height")
                .long("height")
                .short('h')
                .about("Use that many `rows` for the plot")
                .default_value("40")
                .takes_value(true)
        );
    plot = add_common_options(plot);

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
                .takes_value(true)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("Be more verbose")
                .takes_value(false)
        )
        .subcommand(hist)
        .subcommand(plot)
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
        let arg_vec = vec!["lowcharts", "plot", "--max", "1.1", "-m", "0.9", "--height", "11"];
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
}
