//! TODO

/// Translates IDs from the evolution basis into IDs using PDG Monte Carlo IDs.
#[must_use]
pub fn evol_to_pdg_mc_ids(id: i32) -> Vec<(i32, f64)> {
    match id {
        100 => vec![
            (2, 1.0),
            (-2, 1.0),
            (1, 1.0),
            (-1, 1.0),
            (3, 1.0),
            (-3, 1.0),
            (4, 1.0),
            (-4, 1.0),
            (5, 1.0),
            (-5, 1.0),
            (6, 1.0),
            (-6, 1.0),
        ],
        103 => vec![(2, 1.0), (-2, 1.0), (1, -1.0), (-1, -1.0)],
        108 => vec![
            (2, 1.0),
            (-2, 1.0),
            (1, 1.0),
            (-1, 1.0),
            (3, -2.0),
            (-3, -2.0),
        ],
        115 => vec![
            (2, 1.0),
            (-2, 1.0),
            (1, 1.0),
            (-1, 1.0),
            (3, 1.0),
            (-3, 1.0),
            (4, -3.0),
            (-4, -3.0),
        ],
        124 => vec![
            (2, 1.0),
            (-2, 1.0),
            (1, 1.0),
            (-1, 1.0),
            (3, 1.0),
            (-3, 1.0),
            (4, 1.0),
            (-4, 1.0),
            (5, -4.0),
            (-5, -4.0),
        ],
        135 => vec![
            (2, 1.0),
            (-2, 1.0),
            (1, 1.0),
            (-1, 1.0),
            (3, 1.0),
            (-3, 1.0),
            (4, 1.0),
            (-4, 1.0),
            (5, 1.0),
            (-5, 1.0),
            (6, -5.0),
            (-6, -5.0),
        ],
        200 => vec![
            (1, 1.0),
            (-1, -1.0),
            (2, 1.0),
            (-2, -1.0),
            (3, 1.0),
            (-3, -1.0),
            (4, 1.0),
            (-4, -1.0),
            (5, 1.0),
            (-5, -1.0),
            (6, 1.0),
            (-6, -1.0),
        ],
        203 => vec![(2, 1.0), (-2, -1.0), (1, -1.0), (-1, 1.0)],
        208 => vec![
            (2, 1.0),
            (-2, -1.0),
            (1, 1.0),
            (-1, -1.0),
            (3, -2.0),
            (-3, 2.0),
        ],
        215 => vec![
            (2, 1.0),
            (-2, -1.0),
            (1, 1.0),
            (-1, -1.0),
            (3, 1.0),
            (-3, -1.0),
            (4, -3.0),
            (-4, 3.0),
        ],
        224 => vec![
            (2, 1.0),
            (-2, -1.0),
            (1, 1.0),
            (-1, -1.0),
            (3, 1.0),
            (-3, -1.0),
            (4, 1.0),
            (-4, -1.0),
            (5, -4.0),
            (-5, 4.0),
        ],
        235 => vec![
            (2, 1.0),
            (-2, -1.0),
            (1, 1.0),
            (-1, -1.0),
            (3, 1.0),
            (-3, -1.0),
            (4, 1.0),
            (-4, -1.0),
            (5, 1.0),
            (-5, -1.0),
            (6, -5.0),
            (-6, 5.0),
        ],
        _ => vec![(id, 1.0)],
    }
}

/// Return the charge-conjugated PDG ID of `pid`.
#[must_use]
pub const fn charge_conjugate_pdg_pid(pid: i32) -> i32 {
    match pid {
        21 | 22 => pid,
        _ => -pid,
    }
}

/// Return the charge-conjugated particle ID of `pid` for the basis `lumi_id_types`. The returned
/// tuple contains a factor that possible arises during the carge conjugation.
#[must_use]
pub fn charge_conjugate(lumi_id_types: &str, pid: i32) -> (i32, f64) {
    match (lumi_id_types, pid) {
        ("pdg_mc_ids", _) => (charge_conjugate_pdg_pid(pid), 1.0),
        ("evol", 100 | 103 | 108 | 115 | 124 | 135) => (pid, 1.0),
        ("evol", 200 | 203 | 208 | 215 | 224 | 235) => (pid, -1.0),
        ("evol", _) => (charge_conjugate_pdg_pid(pid), 1.0),
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // check photon
        assert_eq!(evol_to_pdg_mc_ids(21), [(21, 1.0)]);

        // check gluon
        assert_eq!(evol_to_pdg_mc_ids(22), [(22, 1.0)]);

        // check singlet
        assert_eq!(
            evol_to_pdg_mc_ids(100),
            [
                (2, 1.0),
                (-2, 1.0),
                (1, 1.0),
                (-1, 1.0),
                (3, 1.0),
                (-3, 1.0),
                (4, 1.0),
                (-4, 1.0),
                (5, 1.0),
                (-5, 1.0),
                (6, 1.0),
                (-6, 1.0),
            ]
        );

        // check T3
        assert_eq!(
            evol_to_pdg_mc_ids(103),
            [(2, 1.0), (-2, 1.0), (1, -1.0), (-1, -1.0)]
        );

        // check T8
        assert_eq!(
            evol_to_pdg_mc_ids(108),
            [
                (2, 1.0),
                (-2, 1.0),
                (1, 1.0),
                (-1, 1.0),
                (3, -2.0),
                (-3, -2.0),
            ],
        );

        // check T15
        assert_eq!(
            evol_to_pdg_mc_ids(115),
            [
                (2, 1.0),
                (-2, 1.0),
                (1, 1.0),
                (-1, 1.0),
                (3, 1.0),
                (-3, 1.0),
                (4, -3.0),
                (-4, -3.0),
            ],
        );

        // check T24
        assert_eq!(
            evol_to_pdg_mc_ids(124),
            [
                (2, 1.0),
                (-2, 1.0),
                (1, 1.0),
                (-1, 1.0),
                (3, 1.0),
                (-3, 1.0),
                (4, 1.0),
                (-4, 1.0),
                (5, -4.0),
                (-5, -4.0),
            ],
        );

        // check T35
        assert_eq!(
            evol_to_pdg_mc_ids(135),
            [
                (2, 1.0),
                (-2, 1.0),
                (1, 1.0),
                (-1, 1.0),
                (3, 1.0),
                (-3, 1.0),
                (4, 1.0),
                (-4, 1.0),
                (5, 1.0),
                (-5, 1.0),
                (6, -5.0),
                (-6, -5.0),
            ],
        );

        // check valence
        assert_eq!(
            evol_to_pdg_mc_ids(200),
            [
                (1, 1.0),
                (-1, -1.0),
                (2, 1.0),
                (-2, -1.0),
                (3, 1.0),
                (-3, -1.0),
                (4, 1.0),
                (-4, -1.0),
                (5, 1.0),
                (-5, -1.0),
                (6, 1.0),
                (-6, -1.0),
            ],
        );

        // check V3
        assert_eq!(
            evol_to_pdg_mc_ids(203),
            [(2, 1.0), (-2, -1.0), (1, -1.0), (-1, 1.0)],
        );

        // check V8
        assert_eq!(
            evol_to_pdg_mc_ids(208),
            [
                (2, 1.0),
                (-2, -1.0),
                (1, 1.0),
                (-1, -1.0),
                (3, -2.0),
                (-3, 2.0),
            ],
        );

        // check V15
        assert_eq!(
            evol_to_pdg_mc_ids(215),
            [
                (2, 1.0),
                (-2, -1.0),
                (1, 1.0),
                (-1, -1.0),
                (3, 1.0),
                (-3, -1.0),
                (4, -3.0),
                (-4, 3.0),
            ],
        );

        // check V24
        assert_eq!(
            evol_to_pdg_mc_ids(224),
            [
                (2, 1.0),
                (-2, -1.0),
                (1, 1.0),
                (-1, -1.0),
                (3, 1.0),
                (-3, -1.0),
                (4, 1.0),
                (-4, -1.0),
                (5, -4.0),
                (-5, 4.0),
            ],
        );

        // check V35
        assert_eq!(
            evol_to_pdg_mc_ids(235),
            [
                (2, 1.0),
                (-2, -1.0),
                (1, 1.0),
                (-1, -1.0),
                (3, 1.0),
                (-3, -1.0),
                (4, 1.0),
                (-4, -1.0),
                (5, 1.0),
                (-5, -1.0),
                (6, -5.0),
                (-6, 5.0),
            ],
        );
    }
}
