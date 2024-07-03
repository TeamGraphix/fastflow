//! Rust-side input validations.
//!
//! # Note
//!
//! - Internal use only, as they're assumed to be validated already.
//! - DO NOT rely on these validations!

use crate::common::{Graph, Nodes};

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
