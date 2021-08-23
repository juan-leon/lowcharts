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
    // Stdin has timestamps
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("timehist")
        .arg("--intervals")
        .arg("2")
        .arg("--duration")
        .arg("200ms")
        .write_stdin("foo 1619655527.888165 bar\nfoo 1619655528.888165 bar\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Not enough data to process"));
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
fn test_splittime() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("split-timehist")
        .arg("1")
        .arg("2")
        .arg("3")
        .arg("4")
        .arg("5")
        .arg("6")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Only 5 different sub-groups are supported",
        ));
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("split-timehist")
        .arg("A")
        .arg("B")
        .arg("C")
        .arg("--intervals")
        .arg("2")
        .write_stdin("1619655527.888165 A\n1619655528.888165 A\n1619655527.888165 B\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Matches: 3."))
        .stdout(predicate::str::contains("A: 2"))
        .stdout(predicate::str::contains("B: 1."))
        .stdout(predicate::str::contains("C: 0."))
        .stdout(predicate::str::contains("Each ∎ represents a count of 1\n"))
        .stdout(predicate::str::contains("[00:18:47.888165] [1/1/0] ∎∎\n"))
        .stdout(predicate::str::contains("[00:18:48.388165] [1/0/0] ∎\n"));
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
                .stdout(predicate::str::contains("\n[3.250]    ●"))
                .stdout(predicate::str::contains("\n[2.500]   ●"))
                .stdout(predicate::str::contains("\n[1.750]  ●"))
                .stdout(predicate::str::contains("\n[1.000] ●"))
                .stderr(predicate::str::contains("[DEBUG] Cannot parse float"));
        }
        Err(_) => assert!(false, "Could not create temp file"),
    }
}

#[test]
fn test_hist_negative_min() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    cmd.arg("hist")
        .arg("--min")
        .arg("-1")
        .arg("--max")
        .arg("10.1")
        .write_stdin("4.2\n2.4\n-2\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Samples = 2; Min = 2.4; Max = 4.2",
        ));
}

#[test]
fn test_common() {
    let mut cmd = Command::cargo_bin("lowcharts").unwrap();
    match NamedTempFile::new() {
        Ok(ref mut file) => {
            writeln!(file, "foo").unwrap();
            writeln!(file, "x").unwrap();
            writeln!(file, "foo").unwrap();
            writeln!(file, "x").unwrap();
            writeln!(file, "foo").unwrap();
            cmd.arg("--color")
                .arg("no")
                .arg("common-terms")
                .arg(file.path().to_str().unwrap())
                .arg("--lines")
                .arg("4")
                .assert()
                .success()
                .stdout(predicate::str::contains("Each ∎ represents a count of 1\n"))
                .stdout(predicate::str::contains("\n[foo] [3] ∎∎∎\n"))
                .stdout(predicate::str::contains("\n[  x] [2] ∎∎\n"));
        }
        Err(_) => assert!(false, "Could not create temp file"),
    }
}
