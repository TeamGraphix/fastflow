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
