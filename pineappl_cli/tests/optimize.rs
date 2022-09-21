use assert_cmd::Command;
use assert_fs::NamedTempFile;

const HELP_STR: &str = "pineappl-optimize 
Optimizes the internal data structure to minimize memory usage

USAGE:
    pineappl optimize [OPTIONS] <INPUT> <OUTPUT>

ARGS:
    <INPUT>     Path to the input grid
    <OUTPUT>    Path to the optimized PineAPPL file

OPTIONS:
        --fk-table <ASSUMPTIONS>    [possible values: Nf6Ind, Nf6Sym, Nf5Ind, Nf5Sym, Nf4Ind,
                                    Nf4Sym, Nf3Ind, Nf3Sym]
    -h, --help                      Print help information
";

#[test]
fn help() {
    Command::cargo_bin("pineappl")
        .unwrap()
        .args(&["optimize", "--help"])
        .assert()
        .success()
        .stdout(HELP_STR);
}

#[test]
fn default() {
    let output = NamedTempFile::new("optimized.pineappl.lz4").unwrap();

    Command::cargo_bin("pineappl")
        .unwrap()
        .args(&[
            "optimize",
            "data/LHCB_WP_7TEV.pineappl.lz4",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("");
}