use super::helpers::{self, GlobalConfiguration, Subcommand};
use anyhow::{bail, Result};
use clap::{ArgGroup, Parser, ValueHint};
use pineappl::bin::BinRemapper;
use std::path::PathBuf;
use std::process::ExitCode;

/// Sums two or more bins of a grid together.
#[derive(Parser)]
#[command(group = ArgGroup::new("mode").required(true))]
pub struct Opts {
    /// Path to the input grid.
    #[arg(value_hint = ValueHint::FilePath)]
    input: PathBuf,
    /// Path to the modified PineAPPL file.
    #[arg(value_hint = ValueHint::FilePath)]
    output: PathBuf,
    /// Sums all bins into a single bin.
    #[arg(long, group = "mode")]
    integrated: bool,
}

impl Subcommand for Opts {
    fn run(&self, _: &GlobalConfiguration) -> Result<ExitCode> {
        let mut grid = helpers::read_grid(&self.input)?;

        if self.integrated {
            if grid.merge_bins(0..grid.bin_info().bins()).is_err() {
                bail!("TODO");
            }
            grid.set_remapper(
                BinRemapper::new(vec![1.0], vec![(0.0, 1.0)]).unwrap_or_else(|_| unreachable!()),
            )?;

            let dimensions = grid.bin_info().dimensions();
            let key_values = grid.key_values_mut();
            for dim in 0..dimensions {
                key_values.remove(&format!("x{}_label", dim + 1));
                key_values.remove(&format!("x{}_label_tex", dim + 1));
                key_values.remove(&format!("x{}_unit", dim + 1));
            }
            key_values.remove("y_label");
            key_values.remove("y_label_tex");
            key_values.remove("y_unit");
        } else {
            unreachable!();
        }

        helpers::write_grid(&self.output, &grid)
    }
}
