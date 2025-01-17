use super::helpers;
use super::{GlobalConfiguration, Subcommand};
use anyhow::{anyhow, Result};
use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{
    value_parser, Arg, ArgAction, ArgMatches, Args, Command, Error, FromArgMatches, Parser,
    ValueHint,
};
use pineappl::bin::BinRemapper;
use pineappl::fk_table::{FkAssumptions, FkTable};
use pineappl::lumi::LumiEntry;
use pineappl::pids;
use std::fs;
use std::ops::{Deref, RangeInclusive};
use std::path::PathBuf;
use std::process::ExitCode;

/// Write a grid modified by various operations.
#[derive(Parser)]
pub struct Opts {
    /// Path to the input grid.
    #[arg(value_hint = ValueHint::FilePath)]
    input: PathBuf,
    /// Path of the modified PineAPPL file.
    #[arg(value_hint = ValueHint::FilePath)]
    output: PathBuf,
    #[command(flatten)]
    more_args: MoreArgs,
}

#[derive(Clone)]
enum OpsArg {
    Cc1(bool),
    Cc2(bool),
    DedupChannels(i64),
    DeleteBins(Vec<RangeInclusive<usize>>),
    DeleteKey(String),
    MergeBins(Vec<RangeInclusive<usize>>),
    Optimize(bool),
    OptimizeFkTable(FkAssumptions),
    Remap(String),
    RemapNorm(f64),
    RemapNormIgnore(Vec<usize>),
    RewriteChannel((usize, LumiEntry)),
    Scale(f64),
    ScaleByBin(Vec<f64>),
    ScaleByOrder(Vec<f64>),
    SetKeyFile(Vec<String>),
    SetKeyValue(Vec<String>),
    SplitLumi(bool),
    Upgrade(bool),
}

struct MoreArgs {
    args: Vec<OpsArg>,
}

impl FromArgMatches for MoreArgs {
    fn from_arg_matches(_: &ArgMatches) -> Result<Self, Error> {
        unreachable!()
    }

    fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, Error> {
        let mut args = Vec::new();
        let ids: Vec<_> = matches.ids().map(|id| id.as_str().to_string()).collect();

        for id in ids {
            let indices: Vec<_> = matches.indices_of(&id).unwrap().collect();
            args.resize(indices.iter().max().unwrap() + 1, None);

            match id.as_str() {
                "cc1" | "cc2" | "optimize" | "split_lumi" | "upgrade" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();
                    assert_eq!(arguments.len(), indices.len());

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        assert_eq!(arg.len(), 1);
                        args[index] = Some(match id.as_str() {
                            "cc1" => OpsArg::Cc1(arg[0]),
                            "cc2" => OpsArg::Cc2(arg[0]),
                            "optimize" => OpsArg::Optimize(arg[0]),
                            "split_lumi" => OpsArg::SplitLumi(arg[0]),
                            "upgrade" => OpsArg::Upgrade(arg[0]),
                            _ => unreachable!(),
                        });
                    }
                }
                "remap_norm_ignore" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        args[index] = Some(match id.as_str() {
                            "remap_norm_ignore" => OpsArg::RemapNormIgnore(arg),
                            _ => unreachable!(),
                        });
                    }
                }
                "dedup_channels" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        assert_eq!(arg.len(), 1);
                        args[index] = Some(match id.as_str() {
                            "dedup_channels" => OpsArg::DedupChannels(arg[0]),
                            _ => unreachable!(),
                        });
                    }
                }
                "delete_key" | "remap" => {
                    let arguments: Vec<Vec<String>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, mut arg) in indices.into_iter().zip(arguments.into_iter()) {
                        assert_eq!(arg.len(), 1);
                        args[index] = Some(match id.as_str() {
                            "delete_key" => OpsArg::DeleteKey(arg.pop().unwrap()),
                            "remap" => OpsArg::Remap(arg.pop().unwrap()),
                            _ => unreachable!(),
                        });
                    }
                }
                "delete_bins" | "merge_bins" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        args[index] = Some(match id.as_str() {
                            "delete_bins" => OpsArg::DeleteBins(arg),
                            "merge_bins" => OpsArg::MergeBins(arg),
                            _ => unreachable!(),
                        });
                    }
                }
                "optimize_fk_table" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        assert_eq!(arg.len(), 1);
                        args[index] = Some(match id.as_str() {
                            "optimize_fk_table" => OpsArg::OptimizeFkTable(arg[0]),
                            _ => unreachable!(),
                        });
                    }
                }
                "remap_norm" | "scale" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        assert_eq!(arg.len(), 1);
                        args[index] = Some(match id.as_str() {
                            "remap_norm" => OpsArg::RemapNorm(arg[0]),
                            "scale" => OpsArg::Scale(arg[0]),
                            _ => unreachable!(),
                        });
                    }
                }
                "rewrite_channel" => {
                    let arguments: Vec<Vec<String>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        assert_eq!(arg.len(), 2);

                        args[index] = Some(OpsArg::RewriteChannel((
                            str::parse(&arg[0]).unwrap(),
                            str::parse(&arg[1]).unwrap(),
                        )));
                    }
                }
                "scale_by_bin" | "scale_by_order" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        args[index] = Some(match id.as_str() {
                            "scale_by_bin" => OpsArg::ScaleByBin(arg),
                            "scale_by_order" => OpsArg::ScaleByOrder(arg),
                            _ => unreachable!(),
                        });
                    }
                }
                "set_key_file" | "set_key_value" => {
                    let arguments: Vec<Vec<_>> = matches
                        .remove_occurrences(&id)
                        .unwrap()
                        .map(Iterator::collect)
                        .collect();

                    for (index, arg) in indices.into_iter().zip(arguments.into_iter()) {
                        args[index] = Some(match id.as_str() {
                            "set_key_file" => OpsArg::SetKeyFile(arg),
                            "set_key_value" => OpsArg::SetKeyValue(arg),
                            _ => unreachable!(),
                        });
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(Self {
            args: args.into_iter().flatten().collect(),
        })
    }

    fn update_from_arg_matches(&mut self, _: &ArgMatches) -> Result<(), Error> {
        unreachable!()
    }

    fn update_from_arg_matches_mut(&mut self, _: &mut ArgMatches) -> Result<(), Error> {
        unreachable!()
    }
}

impl Args for MoreArgs {
    fn augment_args(cmd: Command) -> Command {
        cmd.arg(
            Arg::new("cc1")
                .action(ArgAction::Append)
                .default_missing_value("true")
                .help("Charge conjugate the first initial state")
                .long("cc1")
                .num_args(0..=1)
                .require_equals(true)
                .value_name("ENABLE")
                .value_parser(clap::value_parser!(bool)),
        )
        .arg(
            Arg::new("cc2")
                .action(ArgAction::Append)
                .default_missing_value("true")
                .help("Charge conjugate the second initial state")
                .long("cc2")
                .num_args(0..=1)
                .require_equals(true)
                .value_name("ENABLE")
                .value_parser(clap::value_parser!(bool)),
        )
        .arg(
            Arg::new("dedup_channels")
                .action(ArgAction::Append)
                .default_missing_value("64")
                .help("Deduplicate channels assuming numbers differing by ULPS are the same")
                .long("dedup-channels")
                .num_args(0..=1)
                .require_equals(true)
                .value_name("ULPS")
                .value_parser(value_parser!(i64)),
        )
        .arg(
            Arg::new("delete_bins")
                .action(ArgAction::Append)
                .help("Delete bins with the specified indices")
                .long("delete-bins")
                .num_args(1)
                .value_delimiter(',')
                .value_name("BIN1-BIN2,...")
                .value_parser(helpers::parse_integer_range),
        )
        .arg(
            Arg::new("delete_key")
                .action(ArgAction::Append)
                .help("Delete an internal key-value pair")
                .long("delete-key")
                .value_name("KEY"),
        )
        .arg(
            Arg::new("merge_bins")
                .action(ArgAction::Append)
                .help("Merge specific bins together")
                .long("merge-bins")
                .num_args(1)
                .value_delimiter(',')
                .value_name("BIN1-BIN2,...")
                .value_parser(helpers::parse_integer_range),
        )
        .arg(
            Arg::new("optimize")
                .action(ArgAction::Append)
                .default_missing_value("true")
                .help("Optimize internal data structure to minimize memory and disk usage")
                .long("optimize")
                .num_args(0..=1)
                .require_equals(true)
                .value_name("ENABLE")
                .value_parser(clap::value_parser!(bool)),
        )
        .arg(
            Arg::new("optimize_fk_table")
                .action(ArgAction::Append)
                .help("Optimize internal data structure of an FkTable to minimize memory and disk usage")
                .long("optimize-fk-table")
                .num_args(1)
                .value_name("OPTIMI")
                .value_parser(
                    PossibleValuesParser::new([
                        "Nf6Ind", "Nf6Sym", "Nf5Ind", "Nf5Sym", "Nf4Ind", "Nf4Sym", "Nf3Ind",
                        "Nf3Sym",
                    ])
                    .try_map(|s| s.parse::<FkAssumptions>()),
                ),
        )
        .arg(
            Arg::new("remap")
                .action(ArgAction::Append)
                .help("Modify the bin dimensions and widths")
                .long("remap")
                .value_name("REMAPPING"),
        )
        .arg(
            Arg::new("remap_norm")
                .action(ArgAction::Append)
                .help("Modify the bin normalizations with a common factor")
                .long("remap-norm")
                .value_delimiter(',')
                .value_name("NORM")
                .value_parser(value_parser!(f64)),
        )
        .arg(
            Arg::new("remap_norm_ignore")
                .action(ArgAction::Append)
                .help("Modify the bin normalizations by multiplying with the bin lengths for the given dimensions")
                .long("remap-norm-ignore")
                .num_args(1)
                .value_delimiter(',')
                .value_name("DIM1,...")
                .value_parser(value_parser!(usize)),
        )
        .arg(
            Arg::new("rewrite_channel")
                .action(ArgAction::Append)
                .help("Rewrite the definition of the channel with index IDX")
                .long("rewrite-channel")
                .num_args(2)
                .value_names(["IDX", "CHAN"])
        )
        .arg(
            Arg::new("scale")
                .action(ArgAction::Append)
                .help("Scales all grids with the given factor")
                .long("scale")
                .short('s')
                .value_name("SCALE")
                .value_parser(value_parser!(f64)),
        )
        .arg(
            Arg::new("scale_by_bin")
                .action(ArgAction::Append)
                .help("Scale each bin with a different factor")
                .long("scale-by-bin")
                .num_args(1)
                .value_delimiter(',')
                .value_name("BIN1,BIN2,...")
                .value_parser(value_parser!(f64)),
        )
        .arg(
            Arg::new("scale_by_order")
                .action(ArgAction::Append)
                .help("Scales all grids with order-dependent factors")
                .long("scale-by-order")
                .num_args(1)
                .value_delimiter(',')
                .value_name("AS,AL,LR,LF")
                .value_parser(value_parser!(f64)),
        )
        .arg(
            Arg::new("set_key_value")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .help("Set an internal key-value pair")
                .long("set-key-value")
                .num_args(2)
                .value_names(["KEY", "VALUE"]),
        )
        .arg(
            Arg::new("set_key_file")
                .action(ArgAction::Append)
                .allow_hyphen_values(true)
                .help("Set an internal key-value pair, with value being read from a file")
                .long("set-key-file")
                .num_args(2)
                .value_names(["KEY", "FILE"]),
        )
        .arg(
            Arg::new("split_lumi")
                .action(ArgAction::Append)
                .default_missing_value("true")
                .help("Split the grid such that the luminosity function contains only a single combination per channel")
                .long("split-lumi")
                .num_args(0..=1)
                .require_equals(true)
                .value_name("ENABLE")
                .value_parser(clap::value_parser!(bool)),
        )
        .arg(
            Arg::new("upgrade")
                .action(ArgAction::Append)
                .default_missing_value("true")
                .help("Convert the file format to the most recent version")
                .long("upgrade")
                .num_args(0..=1)
                .require_equals(true)
                .value_name("ENABLE")
                .value_parser(clap::value_parser!(bool)),
        )
    }

    fn augment_args_for_update(_: Command) -> Command {
        unreachable!()
    }
}

impl Subcommand for Opts {
    fn run(&self, _: &GlobalConfiguration) -> Result<ExitCode> {
        let mut grid = helpers::read_grid(&self.input)?;

        for arg in &self.more_args.args {
            match arg {
                OpsArg::Cc1(true) | OpsArg::Cc2(true) => {
                    let cc1 = matches!(arg, OpsArg::Cc1(true));
                    let cc2 = matches!(arg, OpsArg::Cc2(true));

                    let lumi_id_types = grid.key_values().map_or("pdg_mc_ids", |kv| {
                        kv.get("lumi_id_types").map_or("pdg_mc_ids", Deref::deref)
                    });
                    let lumis = grid
                        .lumi()
                        .iter()
                        .map(|entry| {
                            LumiEntry::new(
                                entry
                                    .entry()
                                    .iter()
                                    .map(|&(a, b, f)| {
                                        let (ap, f1) = if cc1 {
                                            pids::charge_conjugate(lumi_id_types, a)
                                        } else {
                                            (a, 1.0)
                                        };
                                        let (bp, f2) = if cc2 {
                                            pids::charge_conjugate(lumi_id_types, b)
                                        } else {
                                            (b, 1.0)
                                        };
                                        (ap, bp, f * f1 * f2)
                                    })
                                    .collect(),
                            )
                        })
                        .collect();

                    let mut initial_state_1 = grid.initial_state_1();
                    let mut initial_state_2 = grid.initial_state_2();

                    if cc1 {
                        initial_state_1 = pids::charge_conjugate_pdg_pid(initial_state_1);
                    }
                    if cc2 {
                        initial_state_2 = pids::charge_conjugate_pdg_pid(initial_state_2);
                    }

                    grid.set_key_value("initial_state_1", &initial_state_1.to_string());
                    grid.set_key_value("initial_state_2", &initial_state_2.to_string());
                    grid.set_lumis(lumis);
                }
                OpsArg::DedupChannels(ulps) => {
                    grid.dedup_channels(*ulps);
                }
                OpsArg::DeleteBins(ranges) => {
                    grid.delete_bins(&ranges.iter().flat_map(|r| r.clone()).collect::<Vec<_>>())
                }
                OpsArg::DeleteKey(key) => {
                    grid.key_values_mut().remove(key);
                }
                OpsArg::MergeBins(ranges) => {
                    // TODO: sort after increasing start indices
                    for range in ranges.iter().rev() {
                        grid.merge_bins(*range.start()..(range.end() + 1))?;
                    }
                }
                OpsArg::Remap(remapping) => grid.set_remapper(str::parse(remapping)?)?,
                OpsArg::RemapNorm(factor) => {
                    let remapper = grid
                        .remapper()
                        .ok_or_else(|| anyhow!("grid does not have a remapper"))?;
                    let normalizations = remapper
                        .normalizations()
                        .iter()
                        .copied()
                        .map(|value| factor * value)
                        .collect();

                    grid.set_remapper(
                        BinRemapper::new(normalizations, remapper.limits().to_vec()).unwrap(),
                    )?;
                }
                OpsArg::RemapNormIgnore(dimensions) => {
                    let remapper = grid
                        .remapper()
                        .ok_or_else(|| anyhow!("grid does not have a remapper"))?;
                    let normalizations = remapper
                        .limits()
                        .chunks_exact(remapper.dimensions())
                        .zip(remapper.normalizations())
                        .map(|(limits, normalization)| {
                            normalization
                                / dimensions
                                    .iter()
                                    .map(|&index| limits[index].1 - limits[index].0)
                                    .product::<f64>()
                        })
                        .collect();

                    grid.set_remapper(
                        BinRemapper::new(normalizations, remapper.limits().to_vec()).unwrap(),
                    )?;
                }
                OpsArg::RewriteChannel((index, new_channel)) => {
                    let mut channels = grid.lumi().to_vec();
                    // TODO: check that `index` is valid
                    channels[*index] = new_channel.clone();
                    grid.set_lumis(channels);
                }
                OpsArg::Scale(factor) => grid.scale(*factor),
                OpsArg::Optimize(true) => grid.optimize(),
                OpsArg::OptimizeFkTable(assumptions) => {
                    let mut fk_table = FkTable::try_from(grid)?;
                    fk_table.optimize(*assumptions);
                    grid = fk_table.into_grid();
                }
                OpsArg::ScaleByBin(factors) => grid.scale_by_bin(factors),
                OpsArg::ScaleByOrder(factors) => {
                    grid.scale_by_order(factors[0], factors[1], factors[2], factors[3], 1.0);
                }
                OpsArg::SetKeyValue(key_value) => {
                    grid.set_key_value(&key_value[0], &key_value[1]);
                }
                OpsArg::SetKeyFile(key_file) => {
                    grid.set_key_value(&key_file[0], &fs::read_to_string(&key_file[1])?);
                }
                OpsArg::SplitLumi(true) => grid.split_lumi(),
                OpsArg::Upgrade(true) => grid.upgrade(),
                OpsArg::Cc1(false)
                | OpsArg::Cc2(false)
                | OpsArg::Optimize(false)
                | OpsArg::SplitLumi(false)
                | OpsArg::Upgrade(false) => {}
            }
        }

        helpers::write_grid(&self.output, &grid)
    }
}
