use std::io::Write;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn test_help_works() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("help"));
}

#[test]
fn test_no_subcommand_fails() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("requires a subcommand"));
}

#[test]
fn test_one_subcommand_help() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("hist")
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("hist"));
}

#[test]
fn test_timehist() {
    // Stdin is closed
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("timehist")
        .assert()
        .success()
        .stderr(predicate::str::contains("Not enough data to process"));
    // Stdin is garbage
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("timehist")
        .write_stdin("foo\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Not enough data to process"))
        .stderr(predicate::str::contains(
            "Could not figure out parsing strategy",
        ));
    // Stdin has timestamps
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("timehist")
        .arg("--intervals")
        .arg("2")
        .write_stdin("foo 1619655527.888165 bar\nfoo 1619655528.888165 bar\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Matches: 2."))
        .stdout(predicate::str::contains("[00:18:47.888165] [1] ∎\n"))
        .stdout(predicate::str::contains("[00:18:48.388165] [1] ∎"));
}

#[test]
fn test_hist() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("hist")
        .arg("--min")
        .arg("1")
        .write_stdin("4.2\n2.4\n0.1\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Samples = 2; Min = 2.4; Max = 4.2",
        ));
}

#[test]
fn test_matchbar() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("matches")
        .arg("-")
        .arg("foo")
        .arg("gnat")
        .arg("bar")
        .write_stdin("foo1\nbar2\nfoo3\nbar4\nfoo5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\n[foo ] [3] ∎∎∎\n"))
        .stdout(predicate::str::contains("\n[gnat] [0] \n"))
        .stdout(predicate::str::contains("\n[bar ] [2] ∎∎\n"));
}

#[test]
fn test_plot() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    match NamedTempFile::new() {
        Ok(ref mut file) => {
            writeln!(file, "1").unwrap();
            writeln!(file, "2").unwrap();
            writeln!(file, "3").unwrap();
            writeln!(file, "4").unwrap();
            writeln!(file, "none").unwrap();
            cmd.arg("--verbose")
                .arg("--color")
                .arg("no")
                .arg("plot")
                .arg(file.path().to_str().unwrap())
                .arg("--height")
                .arg("4")
                .assert()
                .success()
                .stdout(predicate::str::contains("Samples = 4; Min = 1; Max = 4\n"))
                .stdout(predicate::str::contains("\n[3.25]    ●"))
                .stdout(predicate::str::contains("\n[2.5]   ●"))
                .stdout(predicate::str::contains("\n[1.75]  ●"))
                .stdout(predicate::str::contains("\n[1] ●"))
                .stderr(predicate::str::contains("[DEBUG] Cannot parse float"));
        }
        Err(_) => assert!(false, "Could not create temp file"),
    }
}
