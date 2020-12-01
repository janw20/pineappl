use pineappl::bin::BinRemapper;
use pineappl::grid::{Grid, Order};
use pineappl::lagrange_subgrid::LagrangeSubgridV2;
use pineappl::lumi_entry;
use pineappl::subgrid::{ExtraSubgridParams, Subgrid, SubgridParams};
use std::convert::TryFrom;
use std::env;
use std::fs::{self, File};
use yaml_rust::YamlLoader;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        println!("usage: {} <input-yaml> <output-pineappl>", args[0]);
        return;
    }

    let input = &args[1];
    let output = &args[2];

    let yaml = YamlLoader::load_from_str(&fs::read_to_string(input).unwrap()).unwrap();

    assert_eq!(yaml.len(), 1);

    let doc = &yaml[0];

    let interpolation_is_log = doc["interpolation_is_log"].as_bool().unwrap();
    let interpolation_polynomial_degree =
        usize::try_from(doc["interpolation_polynomial_degree"].as_i64().unwrap()).unwrap();
    let interpolation_xgrid = doc["interpolation_xgrid"]
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect::<Vec<_>>();
    let pids = doc["pids"].as_vec().unwrap();

    assert_eq!(interpolation_is_log, false);

    let lepton_pid = 11;
    let lumi: Vec<_> = pids
        .iter()
        .map(|pid| {
            lumi_entry![
                i32::try_from(pid.as_i64().unwrap()).unwrap(),
                lepton_pid,
                1.0
            ]
        })
        .collect();
    // TODO: right now there's only a leading order
    let orders = vec![Order::new(0, 0, 0, 0)];
    let bins = doc["F2total"].as_vec().unwrap().iter().count();
    let bin_limits: Vec<_> = (0..=bins).map(|limit| limit as f64).collect();
    let mut params = SubgridParams::default();

    params.set_reweight(false);
    params.set_x_bins(interpolation_xgrid.len());
    params.set_x_max(*interpolation_xgrid.last().unwrap());
    params.set_x_min(*interpolation_xgrid.first().unwrap());
    params.set_x_order(interpolation_polynomial_degree);

    let mut extra = ExtraSubgridParams::default();

    extra.set_reweight2(false);
    extra.set_x2_bins(1);
    extra.set_x2_max(1.0);
    extra.set_x2_min(1.0);
    extra.set_x2_order(0);

    // TODO: check that the x-grid points are the really the same generated from the f2 function

    let mut grid = Grid::new(lumi, orders, bin_limits, SubgridParams::default());
    let mut limits = vec![];

    // TODO: loop over observables and create subgrids
    for (bin, obs) in doc["F2total"].as_vec().unwrap().iter().enumerate() {
        let q2 = obs["Q2"].as_f64().unwrap();
        let x = obs["x"].as_f64().unwrap();

        limits.push((q2, q2));
        limits.push((x, x));

        // no interpolation in the factorization scale
        params.set_q2_bins(1);
        params.set_q2_max(q2);
        params.set_q2_min(q2);
        params.set_q2_order(0);

        let order = 0;

        for (lumi, values) in obs["values"].as_vec().unwrap().iter().enumerate() {
            let values: Vec<_> = values
                .as_vec()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                // reverse the values, as they are sorted differently in PineAPPL
                .rev()
                .collect();

            assert_eq!(values.len(), params.x_bins());

            if values.iter().any(|v| *v != 0.0) {
                let mut subgrid = LagrangeSubgridV2::new(&params, &extra);
                subgrid.write_q2_slice(0, &values);
                grid.set_subgrid(order, bin, lumi, subgrid.into());
            }
        }
    }

    // set the correct observables
    let normalizations = vec![1.0; bins];
    let remapper = BinRemapper::new(normalizations, limits).unwrap();
    grid.set_remapper(remapper).unwrap();

    // set the initial state PDG ids for the grid
    grid.set_key_value("initial_state_1", "2212");
    grid.set_key_value("initial_state_2", &lepton_pid.to_string());

    grid.write(File::create(output).unwrap()).unwrap();
}