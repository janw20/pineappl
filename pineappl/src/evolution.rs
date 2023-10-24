//! Supporting classes and functions for [`Grid::evolve`].

use super::grid::{Grid, GridError, Order};
use super::import_only_subgrid::ImportOnlySubgridV2;
use super::lumi::LumiEntry;
use super::lumi_entry;
use super::sparse_array3::SparseArray3;
use super::subgrid::{Mu2, Subgrid, SubgridEnum};
use float_cmp::approx_eq;
use itertools::Itertools;
use ndarray::linalg;
use ndarray::{s, Array1, Array2, Array3, ArrayView1, ArrayView4, Axis};
use std::iter;

/// Number of ULPS used to de-duplicate grid values in [`Grid::evolve_info`].
pub(crate) const EVOLVE_INFO_TOL_ULPS: i64 = 64;

/// Number of ULPS used to search for grid values in this module. This value must be a large-enough
/// multiple of [`EVOLVE_INFO_TOL_ULPS`], because otherwise similar values are not found in
/// [`Grid::evolve`]. See <https://github.com/NNPDF/pineappl/issues/223> for details.
const EVOLUTION_TOL_ULPS: i64 = 4 * EVOLVE_INFO_TOL_ULPS;

/// This structure captures the information needed to create an evolution kernel operator (EKO) for
/// a specific [`Grid`].
pub struct EvolveInfo {
    /// Squared factorization scales of the `Grid`.
    pub fac1: Vec<f64>,
    /// Particle identifiers of the `Grid`.
    pub pids1: Vec<i32>,
    /// `x`-grid coordinates of the `Grid`.
    pub x1: Vec<f64>,
    /// Renormalization scales of the `Grid`.
    pub ren1: Vec<f64>,
}

/// Information about the evolution kernel operator (EKO) passed to [`Grid::evolve`] as `operator`,
/// which is used to convert a [`Grid`] into an [`FkTable`]. The dimensions of the EKO must
/// correspond to the values given in [`fac1`], [`pids0`], [`x0`], [`pids1`] and [`x1`], exactly in
/// this order. Members with a `1` are defined at the squared factorization scales given in
/// [`fac1`] (often called process scales) and are found in the [`Grid`] that [`Grid::evolve`] is
/// called with. Members with a `0` are defined at the squared factorization scale [`fac0`] (often
/// called fitting scale or starting scale) and are found in the [`FkTable`] resulting from
/// [`Grid::evolve`].
///
/// The EKO may convert a `Grid` from a basis given by the particle identifiers [`pids1`] to a
/// possibly different basis given by [`pids0`]. This basis must also be identified using
/// [`lumi_id_types`], which tells [`FkTable::convolute`] how to perform a convolution. The members
/// [`ren1`] and [`alphas`] must be the strong couplings given at the respective renormalization
/// scales. Finally, [`xir`] and [`xif`] can be used to vary the renormalization and factorization
/// scales, respectively, around their central values.
///
/// [`FkTable::convolute`]: super::fk_table::FkTable::convolute
/// [`FkTable`]: super::fk_table::FkTable
/// [`alphas`]: Self::alphas
/// [`fac0`]: Self::fac0
/// [`fac1`]: Self::fac1
/// [`lumi_id_types`]: Self::lumi_id_types
/// [`pids0`]: Self::pids0
/// [`pids1`]: Self::pids1
/// [`ren1`]: Self::ren1
/// [`x0`]: Self::x0
/// [`x1`]: Self::x1
/// [`xif`]: Self::xif
/// [`xir`]: Self::xir
pub struct OperatorInfo {
    /// Squared factorization scale of the `FkTable`.
    pub fac0: f64,
    /// Particle identifiers of the `FkTable`.
    pub pids0: Vec<i32>,
    /// `x`-grid coordinates of the `FkTable`
    pub x0: Vec<f64>,
    /// Squared factorization scales of the `Grid`.
    pub fac1: Vec<f64>,
    /// Particle identifiers of the `Grid`. If the `Grid` contains more particle identifiers than
    /// given here, the contributions of them are silently ignored.
    pub pids1: Vec<i32>,
    /// `x`-grid coordinates of the `Grid`.
    pub x1: Vec<f64>,

    /// Renormalization scales of the `Grid`.
    pub ren1: Vec<f64>,
    /// Strong couplings corresponding to the order given in [`ren1`](Self::ren1).
    pub alphas: Vec<f64>,
    /// Multiplicative factor for the central renormalization scale.
    pub xir: f64,
    /// Multiplicative factor for the central factorization scale.
    pub xif: f64,
    /// Identifier of the particle basis for the `FkTable`.
    pub lumi_id_types: String,
}

/// Information about the evolution kernel operator slice (EKO) passed to [`Grid::evolve_slice`] as
/// `operator`, which is used to convert a [`Grid`] into an [`FkTable`]. The dimensions of the EKO
/// must correspond to the values given in [`fac1`], [`pids0`], [`x0`], [`pids1`] and [`x1`],
/// exactly in this order. Members with a `1` are defined at the squared factorization scale given
/// as [`fac1`] (often called process scale) and are found in the [`Grid`] that
/// [`Grid::evolve_slice`] is called with. Members with a `0` are defined at the squared
/// factorization scale [`fac0`] (often called fitting scale or starting scale) and are found in
/// the [`FkTable`] resulting from [`Grid::evolve`].
///
/// The EKO slice may convert a `Grid` from a basis given by the particle identifiers [`pids1`] to
/// a possibly different basis given by [`pids0`]. This basis must also be identified using
/// [`lumi_id_types`], which tells [`FkTable::convolute`] how to perform a convolution. The members
/// [`ren1`] and [`alphas`] must be the strong couplings given at the respective renormalization
/// scales. Finally, [`xir`] and [`xif`] can be used to vary the renormalization and factorization
/// scales, respectively, around their central values.
///
/// [`FkTable::convolute`]: super::fk_table::FkTable::convolute
/// [`FkTable`]: super::fk_table::FkTable
/// [`alphas`]: Self::alphas
/// [`fac0`]: Self::fac0
/// [`fac1`]: Self::fac1
/// [`lumi_id_types`]: Self::lumi_id_types
/// [`pids0`]: Self::pids0
/// [`pids1`]: Self::pids1
/// [`ren1`]: Self::ren1
/// [`x0`]: Self::x0
/// [`x1`]: Self::x1
/// [`xif`]: Self::xif
/// [`xir`]: Self::xir
#[derive(Clone)]
pub struct OperatorSliceInfo {
    /// Squared factorization scale of the `FkTable`.
    pub fac0: f64,
    /// Particle identifiers of the `FkTable`.
    pub pids0: Vec<i32>,
    /// `x`-grid coordinates of the `FkTable`
    pub x0: Vec<f64>,
    /// Squared factorization scale of the slice of `Grid` that should be evolved.
    pub fac1: f64,
    /// Particle identifiers of the `Grid`. If the `Grid` contains more particle identifiers than
    /// given here, the contributions of them are silently ignored.
    pub pids1: Vec<i32>,
    /// `x`-grid coordinates of the `Grid`.
    pub x1: Vec<f64>,

    /// Renormalization scales of the `Grid`.
    pub ren1: Vec<f64>,
    /// Strong couplings corresponding to the order given in [`ren1`](Self::ren1).
    pub alphas: Vec<f64>,
    /// Multiplicative factor for the central renormalization scale.
    pub xir: f64,
    /// Multiplicative factor for the central factorization scale.
    pub xif: f64,
    /// Identifier of the particle basis for the `FkTable`.
    pub lumi_id_types: String,
}

fn gluon_has_pid_zero(grid: &Grid) -> bool {
    grid.key_values()
        .and_then(|key_values| key_values.get("lumi_id_types"))
        .and_then(|value| {
            (value == "pdg_mc_ids").then(|| {
                grid.lumi()
                    .iter()
                    .any(|entry| entry.entry().iter().any(|&(a, b, _)| (a == 0) || (b == 0)))
            })
        })
        .unwrap_or(false)
}

type Pid01IndexTuples = Vec<(usize, usize)>;
type Pid01Tuples = Vec<(i32, i32)>;

fn pid_slices(
    operator: &ArrayView4<f64>,
    info: &OperatorSliceInfo,
    gluon_has_pid_zero: bool,
    pid1_nonzero: &dyn Fn(i32) -> bool,
) -> Result<(Pid01IndexTuples, Pid01Tuples), GridError> {
    // list of all non-zero PID indices
    let pid_indices: Vec<_> = (0..operator.dim().2)
        .cartesian_product(0..operator.dim().0)
        .filter(|&(pid0_idx, pid1_idx)| {
            // 1) at least one element of the operator must be non-zero, and 2) the pid must be
            // contained in the lumi somewhere
            operator
                .slice(s![pid1_idx, .., pid0_idx, ..])
                .iter()
                .any(|&value| value != 0.0)
                && pid1_nonzero(if gluon_has_pid_zero && info.pids1[pid1_idx] == 21 {
                    0
                } else {
                    info.pids1[pid1_idx]
                })
        })
        .collect();

    if pid_indices.is_empty() {
        return Err(GridError::EvolutionFailure(
            "no non-zero operator found; result would be an empty FkTable".to_string(),
        ));
    }

    // list of all non-zero (pid0, pid1) combinations
    let pids = pid_indices
        .iter()
        .map(|&(pid0_idx, pid1_idx)| {
            (
                info.pids0[pid0_idx],
                if gluon_has_pid_zero && info.pids1[pid1_idx] == 21 {
                    0
                } else {
                    info.pids1[pid1_idx]
                },
            )
        })
        .collect();

    Ok((pid_indices, pids))
}

fn lumi0_with_one(pids: &[(i32, i32)]) -> Vec<i32> {
    let mut pids0: Vec<_> = pids.iter().map(|&(pid0, _)| pid0).collect();
    pids0.sort_unstable();
    pids0.dedup();

    pids0
}

fn lumi0_with_two(pids_a: &[(i32, i32)], pids_b: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut pids0_a: Vec<_> = pids_a.iter().map(|&(pid0, _)| pid0).collect();
    pids0_a.sort_unstable();
    pids0_a.dedup();
    let mut pids0_b: Vec<_> = pids_b.iter().map(|&(pid0, _)| pid0).collect();
    pids0_b.sort_unstable();
    pids0_b.dedup();

    pids0_a
        .iter()
        .copied()
        .cartesian_product(pids0_b.iter().copied())
        .collect()
}

fn operator_slices(
    operator: &ArrayView4<f64>,
    info: &OperatorSliceInfo,
    pid_indices: &[(usize, usize)],
    x1: &[f64],
) -> Result<Vec<Array2<f64>>, GridError> {
    // permutation between the grid x values and the operator x1 values
    let x1_indices: Vec<_> = x1
        .iter()
        .map(|&x1p| {
            info.x1
                .iter()
                .position(|&x1| approx_eq!(f64, x1p, x1, ulps = EVOLUTION_TOL_ULPS))
                .ok_or_else(|| {
                    GridError::EvolutionFailure(format!("no operator for x = {x1p} found"))
                })
        })
        // TODO: use `try_collect` once stabilized
        .collect::<Result<_, _>>()?;

    // create the corresponding operators accessible in the form [muf2, x0, x1]
    let operators: Vec<_> = pid_indices
        .iter()
        .map(|&(pid0_idx, pid1_idx)| {
            operator
                .slice(s![pid1_idx, .., pid0_idx, ..])
                .select(Axis(0), &x1_indices)
                .reversed_axes()
                .as_standard_layout()
                .into_owned()
        })
        .collect();

    Ok(operators)
}

type X1aX1bOp2Tuple = (Vec<f64>, Vec<f64>, Array2<f64>);

fn ndarray_from_subgrid_orders_slice(
    info: &OperatorSliceInfo,
    subgrids: &ArrayView1<SubgridEnum>,
    orders: &[Order],
    order_mask: &[bool],
) -> Result<X1aX1bOp2Tuple, GridError> {
    // TODO: skip empty subgrids

    let fac1 = info.xif * info.xif * info.fac1;
    let mut x1_a: Vec<_> = subgrids
        .iter()
        .enumerate()
        .filter(|(index, _)| order_mask.get(*index).copied().unwrap_or(true))
        .flat_map(|(_, subgrid)| subgrid.x1_grid().into_owned())
        .collect();
    let mut x1_b: Vec<_> = subgrids
        .iter()
        .enumerate()
        .filter(|(index, _)| order_mask.get(*index).copied().unwrap_or(true))
        .flat_map(|(_, subgrid)| subgrid.x2_grid().into_owned())
        .collect();

    x1_a.sort_by(f64::total_cmp);
    x1_a.dedup_by(|a, b| approx_eq!(f64, *a, *b, ulps = EVOLUTION_TOL_ULPS));
    x1_b.sort_by(f64::total_cmp);
    x1_b.dedup_by(|a, b| approx_eq!(f64, *a, *b, ulps = EVOLUTION_TOL_ULPS));

    let mut array = Array2::<f64>::zeros((x1_a.len(), x1_b.len()));

    // add subgrids for different orders, but the same bin and lumi, using the right
    // couplings
    for (subgrid, order) in subgrids
        .iter()
        .zip(orders.iter())
        .zip(order_mask.iter().chain(iter::repeat(&true)))
        .filter_map(|((subgrid, order), &enabled)| {
            (enabled && !subgrid.is_empty()).then_some((subgrid, order))
        })
    {
        let mut logs = 1.0;

        if order.logxir > 0 {
            if approx_eq!(f64, info.xir, 1.0, ulps = 4) {
                continue;
            }

            logs *= (info.xir * info.xir).ln();
        }

        if order.logxif > 0 {
            if approx_eq!(f64, info.xif, 1.0, ulps = 4) {
                continue;
            }

            logs *= (info.xif * info.xif).ln();
        }

        // TODO: use `try_collect` once stabilized
        let xa_indices: Vec<_> = subgrid
            .x1_grid()
            .iter()
            .map(|&xa| {
                x1_a.iter()
                    .position(|&x1a| approx_eq!(f64, x1a, xa, ulps = EVOLUTION_TOL_ULPS))
                    .ok_or_else(|| {
                        GridError::EvolutionFailure(format!("no operator for x1 = {xa} found"))
                    })
            })
            .collect::<Result<_, _>>()?;
        let xb_indices: Vec<_> = subgrid
            .x2_grid()
            .iter()
            .map(|&xb| {
                x1_b.iter()
                    .position(|&x1b| approx_eq!(f64, x1b, xb, ulps = EVOLUTION_TOL_ULPS))
                    .ok_or_else(|| {
                        GridError::EvolutionFailure(format!("no operator for x1 = {xb} found"))
                    })
            })
            .collect::<Result<_, _>>()?;

        for ((ifac1, ix1, ix2), value) in subgrid.indexed_iter() {
            let Mu2 { ren, fac } = subgrid.mu2_grid()[ifac1];

            if !approx_eq!(
                f64,
                info.xif * info.xif * fac,
                fac1,
                ulps = EVOLUTION_TOL_ULPS
            ) {
                continue;
            }

            let mur2 = info.xir * info.xir * ren;

            let als = if order.alphas == 0 {
                1.0
            } else if let Some(alphas) =
                info.ren1
                    .iter()
                    .zip(info.alphas.iter())
                    .find_map(|(&ren1, &alphas)| {
                        approx_eq!(f64, ren1, mur2, ulps = EVOLUTION_TOL_ULPS).then(|| alphas)
                    })
            {
                alphas.powi(order.alphas.try_into().unwrap())
            } else {
                return Err(GridError::EvolutionFailure(format!(
                    "no alphas for mur2 = {mur2} found"
                )));
            };

            array[[xa_indices[ix1], xb_indices[ix2]]] += als * logs * value;
        }
    }

    Ok((x1_a, x1_b, array))
}

pub(crate) fn evolve_slice_with_one(
    grid: &Grid,
    operator: &ArrayView4<f64>,
    info: &OperatorSliceInfo,
    order_mask: &[bool],
) -> Result<(Array3<SubgridEnum>, Vec<LumiEntry>), GridError> {
    let gluon_has_pid_zero = gluon_has_pid_zero(grid);
    let has_pdf1 = grid.has_pdf1();

    let (pid_indices, pids) = pid_slices(operator, info, gluon_has_pid_zero, &|pid| {
        grid.lumi()
            .iter()
            .flat_map(LumiEntry::entry)
            .any(|&(a, b, _)| if has_pdf1 { a } else { b } == pid)
    })?;

    let lumi0 = lumi0_with_one(&pids);
    let mut sub_fk_tables = Vec::with_capacity(grid.bin_info().bins() * lumi0.len());
    let new_axis = if has_pdf1 { 2 } else { 1 };

    let mut last_x1 = Vec::new();
    let mut ops = Vec::new();

    for subgrids_ol in grid.subgrids().axis_iter(Axis(1)) {
        let mut tables = vec![Array1::zeros(info.x0.len()); lumi0.len()];

        for (subgrids_o, lumi1) in subgrids_ol.axis_iter(Axis(1)).zip(grid.lumi()) {
            let (x1_a, x1_b, array) =
                ndarray_from_subgrid_orders_slice(info, &subgrids_o, grid.orders(), order_mask)?;

            let x1 = if has_pdf1 { x1_a } else { x1_b };

            if x1.is_empty() {
                continue;
            }

            if (last_x1.len() != x1.len())
                || last_x1
                    .iter()
                    .zip(x1.iter())
                    .any(|(&lhs, &rhs)| !approx_eq!(f64, lhs, rhs, ulps = EVOLUTION_TOL_ULPS))
            {
                ops = operator_slices(operator, info, &pid_indices, &x1)?;
                last_x1 = x1;
            }

            for (&pid1, &factor) in lumi1
                .entry()
                .iter()
                .map(|(a, b, f)| if has_pdf1 { (a, f) } else { (b, f) })
            {
                for (fk_table, op) in
                    lumi0
                        .iter()
                        .zip(tables.iter_mut())
                        .filter_map(|(&pid0, fk_table)| {
                            pids.iter()
                                .zip(ops.iter())
                                .find_map(|(&(p0, p1), op)| {
                                    (p0 == pid0 && p1 == pid1).then_some(op)
                                })
                                .map(|op| (fk_table, op))
                        })
                {
                    fk_table.scaled_add(factor, &op.dot(&array.index_axis(Axis(new_axis - 1), 0)));
                }
            }
        }

        sub_fk_tables.extend(tables.into_iter().map(|table| {
            ImportOnlySubgridV2::new(
                SparseArray3::from_ndarray(
                    table
                        .insert_axis(Axis(0))
                        .insert_axis(Axis(new_axis))
                        .view(),
                    0,
                    1,
                ),
                vec![Mu2 {
                    // TODO: FK tables don't depend on the renormalization scale
                    //ren: -1.0,
                    ren: info.fac0,
                    fac: info.fac0,
                }],
                if has_pdf1 { info.x0.clone() } else { vec![1.0] },
                if has_pdf1 { vec![1.0] } else { info.x0.clone() },
            )
            .into()
        }));
    }

    let pid = if has_pdf1 {
        grid.initial_state_2()
    } else {
        grid.initial_state_1()
    };

    Ok((
        Array1::from_iter(sub_fk_tables)
            .into_shape((1, grid.bin_info().bins(), lumi0.len()))
            .unwrap(),
        lumi0
            .iter()
            .map(|&a| {
                lumi_entry![
                    if has_pdf1 { a } else { pid },
                    if has_pdf1 { pid } else { a },
                    1.0
                ]
            })
            .collect(),
    ))
}

pub(crate) fn evolve_slice_with_two(
    grid: &Grid,
    operator: &ArrayView4<f64>,
    info: &OperatorSliceInfo,
    order_mask: &[bool],
) -> Result<(Array3<SubgridEnum>, Vec<LumiEntry>), GridError> {
    let gluon_has_pid_zero = gluon_has_pid_zero(grid);

    let (pid_indices_a, pids_a) = pid_slices(operator, info, gluon_has_pid_zero, &|pid1| {
        grid.lumi()
            .iter()
            .flat_map(LumiEntry::entry)
            .any(|&(a, _, _)| a == pid1)
    })?;
    let (pid_indices_b, pids_b) = pid_slices(operator, info, gluon_has_pid_zero, &|pid1| {
        grid.lumi()
            .iter()
            .flat_map(LumiEntry::entry)
            .any(|&(_, b, _)| b == pid1)
    })?;

    let lumi0 = lumi0_with_two(&pids_a, &pids_b);
    let mut sub_fk_tables = Vec::with_capacity(grid.bin_info().bins() * lumi0.len());

    let mut last_x1a = Vec::new();
    let mut last_x1b = Vec::new();
    let mut operators_a = Vec::new();
    let mut operators_b = Vec::new();

    for subgrids_ol in grid.subgrids().axis_iter(Axis(1)) {
        let mut tables = vec![Array2::zeros((info.x0.len(), info.x0.len())); lumi0.len()];

        for (subgrids_o, lumi1) in subgrids_ol.axis_iter(Axis(1)).zip(grid.lumi()) {
            let (x1_a, x1_b, array) =
                ndarray_from_subgrid_orders_slice(info, &subgrids_o, grid.orders(), order_mask)?;

            if (last_x1a.len() != x1_a.len())
                || last_x1a
                    .iter()
                    .zip(x1_a.iter())
                    .any(|(&lhs, &rhs)| !approx_eq!(f64, lhs, rhs, ulps = EVOLUTION_TOL_ULPS))
            {
                operators_a = operator_slices(operator, info, &pid_indices_a, &x1_a)?;
                last_x1a = x1_a;
            }

            if (last_x1b.len() != x1_b.len())
                || last_x1b
                    .iter()
                    .zip(x1_b.iter())
                    .any(|(&lhs, &rhs)| !approx_eq!(f64, lhs, rhs, ulps = EVOLUTION_TOL_ULPS))
            {
                operators_b = operator_slices(operator, info, &pid_indices_b, &x1_b)?;
                last_x1b = x1_b;
            }

            let mut tmp = Array2::zeros((last_x1a.len(), info.x0.len()));

            for &(pida1, pidb1, factor) in lumi1.entry() {
                for (fk_table, opa, opb) in
                    lumi0
                        .iter()
                        .zip(tables.iter_mut())
                        .filter_map(|(&(pida0, pidb0), fk_table)| {
                            pids_a
                                .iter()
                                .zip(operators_a.iter())
                                .find_map(|(&(pa0, pa1), opa)| {
                                    (pa0 == pida0 && pa1 == pida1).then_some(opa)
                                })
                                .zip(pids_b.iter().zip(operators_b.iter()).find_map(
                                    |(&(pb0, pb1), opb)| {
                                        (pb0 == pidb0 && pb1 == pidb1).then_some(opb)
                                    },
                                ))
                                .map(|(opa, opb)| (fk_table, opa, opb))
                        })
                {
                    linalg::general_mat_mul(1.0, &array, &opb.t(), 0.0, &mut tmp);
                    linalg::general_mat_mul(factor, opa, &tmp, 1.0, fk_table);
                }
            }
        }

        sub_fk_tables.extend(tables.into_iter().map(|table| {
            ImportOnlySubgridV2::new(
                SparseArray3::from_ndarray(table.insert_axis(Axis(0)).view(), 0, 1),
                vec![Mu2 {
                    // TODO: FK tables don't depend on the renormalization scale
                    //ren: -1.0,
                    ren: info.fac0,
                    fac: info.fac0,
                }],
                info.x0.clone(),
                info.x0.clone(),
            )
            .into()
        }));
    }

    Ok((
        Array1::from_iter(sub_fk_tables)
            .into_shape((1, grid.bin_info().bins(), lumi0.len()))
            .unwrap(),
        lumi0.iter().map(|&(a, b)| lumi_entry![a, b, 1.0]).collect(),
    ))
}
