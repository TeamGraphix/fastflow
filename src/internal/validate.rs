//! Rust-side input validations.
//!
//! # Note
//!
//! - Internal module for testing.

use crate::common::{Layer, Nodes};

/// Checks if the layer-zero nodes are correctly chosen.
///
/// # Arguments
///
/// - `layer`: The layer.
/// - `oset`: The set of output nodes.
/// - `iff`: If `true`, `layer[u] == 0` "iff" `u` is in `oset`. Otherwise "if".
pub fn check_initial(layer: &Layer, oset: &Nodes, iff: bool) -> anyhow::Result<()> {
    for (u, &lu) in layer.iter().enumerate() {
        match (oset.contains(&u), lu == 0) {
            (true, false) => {
                let err = anyhow::anyhow!("initial check failed")
                    .context(format!("layer({u}) != 0 && {u} in O"));
                return Err(err);
            }
            (false, true) if iff => {
                let err = anyhow::anyhow!("initial check failed")
                    .context(format!("layer({u}) == 0 && {u} not in O"));
                return Err(err);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks if the domain of `f` is in `vset - oset` and the codomain is in `vset - iset`.
///
/// # Arguments
///
/// - `f_flatiter`: Flow, gflow, or pflow as `impl Iterator<Item = (&usize, &usize)>`.
/// - `vset`: All nodes.
/// - `iset`: Input nodes.
/// - `oset`: Output nodes.
///
/// # Note
///
/// It is allowed for `f[i]` to contain `i`, even if `i` is in `iset`.
pub fn check_domain<'a, 'b>(
    f_flatiter: impl Iterator<Item = (&'a usize, &'b usize)>,
    vset: &Nodes,
    iset: &Nodes,
    oset: &Nodes,
) -> anyhow::Result<()> {
    let icset = vset - iset;
    let ocset = vset - oset;
    let mut dom = Nodes::new();
    for (&i, &fi) in f_flatiter {
        dom.insert(i);
        if i != fi && !icset.contains(&fi) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{fi} not in V\\I"));
            return Err(err);
        }
    }
    if dom != ocset {
        let err = anyhow::anyhow!("domain check failed")
            .context(format!("invalid domain: {dom:?} != V\\O"));
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::*;
    use crate::common::Nodes;

    #[test]
    fn test_check_initial() {
        let layer = vec![0, 0, 0, 1, 1, 1];
        let oset = Nodes::from([0, 1]);
        check_initial(&layer, &oset, false).unwrap();
    }

    #[test]
    #[should_panic = "initial check failed"]
    fn test_check_initial_ng() {
        let layer = vec![0, 0, 0, 1, 1, 1];
        let oset = Nodes::from([0, 1, 2, 3]);
        check_initial(&layer, &oset, false).unwrap();
    }

    #[test]
    fn test_check_initial_iff() {
        let layer = vec![0, 0, 0, 1, 1, 1];
        let oset = Nodes::from([0, 1, 2]);
        check_initial(&layer, &oset, true).unwrap();
    }

    #[test]
    #[should_panic = "initial check failed"]
    fn test_check_initial_iff_ng() {
        let layer = vec![0, 0, 0, 1, 1, 1];
        let oset = Nodes::from([0, 1]);
        check_initial(&layer, &oset, true).unwrap();
    }

    #[test]
    fn test_check_domain_flow() {
        let f = hashbrown::HashMap::<usize, usize>::from([(0, 1), (1, 2)]);
        let vset = Nodes::from([0, 1, 2]);
        let iset = Nodes::from([0]);
        let oset = Nodes::from([2]);
        check_domain(f.iter(), &vset, &iset, &oset).unwrap();
    }

    #[test]
    fn test_check_domain_gflow() {
        let f = hashbrown::HashMap::<usize, Nodes>::from([
            (0, Nodes::from([0, 1])),
            (1, Nodes::from([2])),
        ]);
        let vset = Nodes::from([0, 1, 2]);
        let iset = Nodes::from([0]);
        let oset = Nodes::from([2]);
        let f_flatiter = f
            .iter()
            .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
        check_domain(f_flatiter, &vset, &iset, &oset).unwrap();
    }
}
