use lhapdf::PdfSet;
use pineappl::grid::Grid;
use prettytable::{cell, row, Table};
use rayon::{prelude::*, ThreadPoolBuilder};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use super::helpers::create_table;

pub fn subcommand(
    input: &str,
    pdfset: &str,
    cl: f64,
    threads: usize,
    orders: &[(u32, u32)],
) -> Result<Table, Box<dyn Error>> {
    let grid = Grid::read(BufReader::new(File::open(input)?))?;
    let set = PdfSet::new(&pdfset.parse().map_or_else(
        |_| pdfset.to_string(),
        |lhaid| lhapdf::lookup_pdf(lhaid).unwrap().0,
    ));
    let pdfs = set.mk_pdfs();

    let orders: Vec<_> = grid
        .orders()
        .iter()
        .map(|order| {
            orders.is_empty()
                || orders
                    .iter()
                    .any(|other| (order.alphas == other.0) && (order.alpha == other.1))
        })
        .collect();

    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    let results: Vec<f64> = pdfs
        .into_par_iter()
        .flat_map(|pdf| {
            grid.convolute(
                &|id, x, q2| pdf.xfx_q2(id, x, q2),
                &|id, x, q2| pdf.xfx_q2(id, x, q2),
                &|q2| pdf.alphas_q2(q2),
                &orders,
                &[],
                &[],
                &[(1.0, 1.0)],
            )
        })
        .collect();

    let bin_info = grid.bin_info();
    let left_limits: Vec<_> = (0..bin_info.dimensions())
        .map(|i| bin_info.left(i))
        .collect();
    let right_limits: Vec<_> = (0..bin_info.dimensions())
        .map(|i| bin_info.right(i))
        .collect();
    let normalizations = bin_info.normalizations();

    let mut title = row![];
    title.add_cell(cell!(c->"bin"));
    for i in 0..bin_info.dimensions() {
        let mut cell = cell!(c->&format!("x{}", i + 1));
        cell.set_hspan(2);
        title.add_cell(cell);
    }
    title.add_cell(cell!(c->"diff"));
    title.add_cell(cell!(c->"integ"));
    title.add_cell(cell!(c->"neg unc"));
    title.add_cell(cell!(c->"pos unc"));

    let mut table = create_table();
    table.set_titles(title);

    for bin in 0..bin_info.bins() {
        let values: Vec<_> = results
            .iter()
            .skip(bin)
            .step_by(bin_info.bins())
            .cloned()
            .collect();
        let uncertainty = set.uncertainty(&values, cl, false);

        let row = table.add_empty_row();

        row.add_cell(cell!(r->&format!("{}", bin)));
        for (left, right) in left_limits.iter().zip(right_limits.iter()) {
            row.add_cell(cell!(r->&format!("{}", left[bin])));
            row.add_cell(cell!(r->&format!("{}", right[bin])));
        }
        row.add_cell(cell!(r->&format!("{:.7e}", uncertainty.central)));
        row.add_cell(cell!(r->&format!("{:.7e}", uncertainty.central * normalizations[bin])));
        row.add_cell(
            cell!(r->&format!("{:.2}%", (-uncertainty.errminus / uncertainty.central) * 100.0)),
        );
        row.add_cell(
            cell!(r->&format!("{:.2}%", (uncertainty.errplus / uncertainty.central) * 100.0)),
        );
    }

    Ok(table)
}
