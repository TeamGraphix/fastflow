//! Rust-side input validations.
//!
//! # Note
//!
//! - Internal module for testing.

use crate::common::{Graph, Layer, Nodes};

/// Checks if the graph is valid.
///
/// # Returns
///
/// Returns `Err` if any of the following conditions are met:
///
/// - `g` is empty.
/// - `g` contains self-loops.
/// - `g` is not symmetric.
/// - `g` contains nodes other than `0..g.len()`.
/// - `iset`/`oset` contains inconsistent nodes.
pub fn check_graph(g: &Graph, iset: &Nodes, oset: &Nodes) -> anyhow::Result<()> {
    let n = g.len();
    if n == 0 {
        anyhow::bail!("empty graph");
    }
    for (u, gu) in g.iter().enumerate() {
        if gu.contains(&u) {
            anyhow::bail!("self-loop detected: {u}");
        }
        gu.iter().try_for_each(|&v| {
            if v >= n {
                anyhow::bail!("node index out of range: {v}");
            }
            if !g[v].contains(&u) {
                anyhow::bail!("g must be undirected: needs {v} -> {u}");
            }
            Ok(())
        })?;
    }
    iset.iter().try_for_each(|&u| {
        if !(0..n).contains(&u) {
            anyhow::bail!("unknown node in iset: {u}");
        }
        Ok(())
    })?;
    oset.iter().try_for_each(|&u| {
        if !(0..n).contains(&u) {
            anyhow::bail!("unknown node in oset: {u}");
        }
        Ok(())
    })?;
    Ok(())
}

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
    for (&i, &fi) in f_flatiter {
        if !ocset.contains(&i) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{i} not in V\\O"));
            return Err(err);
        }
        if i != fi && !icset.contains(&fi) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{fi} not in V\\I"));
            return Err(err);
        }
    }
    Ok(())
}
