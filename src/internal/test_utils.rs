//! Testing utilities.

use std::sync::LazyLock;

use crate::common::{Graph, Nodes};

pub mod exports {
    pub use hashbrown::{HashMap, HashSet};
}

macro_rules! map {
    ($($u:literal: $v:expr),*) => {
        // Dirty .expect to handle i32 -> usize conversion
        $crate::internal::test_utils::exports::HashMap::from_iter([$(($u, ($v).try_into().expect("dynamic coersion"))),*].into_iter())
    };
    ($($u:literal: $v:expr),*,) => {map! { $($u: $v),* }};
}

macro_rules! set {
    ($($u:literal),*) => {
        $crate::internal::test_utils::exports::HashSet::from_iter([$($u),*].into_iter())
    };
    ($($u:literal),*,) => {set! { $($u),* }};
}

/// Creates a undirected graph from edges.
pub fn graph<const N: usize>(edges: &[(usize, usize); N]) -> Graph {
    let n = edges
        .iter()
        .map(|&(u, v)| u.max(v) + 1)
        .max()
        .unwrap_or_default();
    let mut g = vec![Nodes::new(); n];
    for &(u, v) in edges {
        g[u].insert(v);
        g[v].insert(u);
    }
    g
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    pub g: Graph,
    pub iset: Nodes,
    pub oset: Nodes,
}

pub static CASE0: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 1
    TestCase {
        g: graph(&[(0, 1)]),
        iset: Nodes::from([0, 1]),
        oset: Nodes::from([0, 1]),
    }
});

pub static CASE1: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 1 - 2 - 3 - 4
    TestCase {
        g: graph(&[(0, 1), (1, 2), (2, 3), (3, 4)]),
        iset: Nodes::from([0]),
        oset: Nodes::from([4]),
    }
});

pub static CASE2: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 2 - 4
    //     |
    // 1 - 3 - 5
    TestCase {
        g: graph(&[(0, 2), (1, 3), (2, 4), (3, 5), (2, 3)]),
        iset: Nodes::from([0, 1]),
        oset: Nodes::from([4, 5]),
    }
});

pub static CASE3: LazyLock<TestCase> = LazyLock::new(|| {
    //   ______
    //  /      |
    // 0 - 3   |
    //    /    |
    //   /     |
    //  /      |
    // 1 - 4   |
    //  \ /    |
    //   X    /
    //  / \  /
    // 2 - 5
    TestCase {
        g: graph(&[(0, 3), (0, 5), (1, 3), (1, 4), (1, 5), (2, 4), (2, 5)]),
        iset: Nodes::from([0, 1, 2]),
        oset: Nodes::from([3, 4, 5]),
    }
});

pub static CASE4: LazyLock<TestCase> = LazyLock::new(|| {
    //   0 - 1
    //  /|   |
    // 4 |   |
    //  \|   |
    //   2 - 5 - 3
    TestCase {
        g: graph(&[(0, 1), (0, 2), (0, 4), (1, 5), (2, 4), (2, 5), (3, 5)]),
        iset: Nodes::from([0, 1]),
        oset: Nodes::from([4, 5]),
    }
});

pub static CASE5: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 2
    //  \ /
    //   X
    //  / \
    // 1 - 3
    TestCase {
        g: graph(&[(0, 2), (0, 3), (1, 2), (1, 3)]),
        iset: Nodes::from([0, 1]),
        oset: Nodes::from([2, 3]),
    }
});

pub static CASE6: LazyLock<TestCase> = LazyLock::new(|| {
    //     3
    //     |
    //     2
    //     |
    // 0 - 1 - 4
    TestCase {
        g: graph(&[(0, 1), (1, 2), (1, 4), (2, 3)]),
        iset: Nodes::from([0]),
        oset: Nodes::from([4]),
    }
});

pub static CASE7: LazyLock<TestCase> = LazyLock::new(|| {
    // 1   2   3
    // | /     |
    // 0 - - - 4
    TestCase {
        g: graph(&[(0, 1), (0, 2), (0, 4), (3, 4)]),
        iset: Nodes::from([0]),
        oset: Nodes::from([4]),
    }
});

pub static CASE8: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 1 -- 3
    //    \|   /|
    //     |\ / |
    //     | /\ |
    //     2 -- 4
    TestCase {
        g: graph(&[(0, 1), (0, 4), (1, 2), (1, 3), (2, 3), (2, 4), (3, 4)]),
        iset: Nodes::from([0]),
        oset: Nodes::from([3, 4]),
    }
});

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use rstest_reuse::{apply, template};

    use super::*;

    #[template]
    #[rstest]
    fn template_tests(
        #[values(&*CASE0, &*CASE1, &*CASE2, &*CASE3, &*CASE4, &*CASE5, &*CASE6, &*CASE7, &*CASE8)]
        input: &TestCase,
    ) {
    }

    /// Checks if the graph is valid.
    ///
    /// In production code, this check should be done in the Python layer.
    fn check_graph(g: &Graph, iset: &Nodes, oset: &Nodes) {
        let n = g.len();
        assert_ne!(n, 0, "empty graph");
        for (u, gu) in g.iter().enumerate() {
            assert!(!gu.contains(&u), "self-loop detected: {u}");
            gu.iter().for_each(|&v| {
                assert!(v < n, "node index out of range: {v}");
                assert!(g[v].contains(&u), "g must be undirected: {u} -> {v}");
            });
        }
        iset.iter().for_each(|&u| {
            assert!((0..n).contains(&u), "unknown node in iset: {u}");
        });
        oset.iter().for_each(|&u| {
            assert!((0..n).contains(&u), "unknown node in oset: {u}");
        });
    }

    #[apply(template_tests)]
    fn test_input(input: &TestCase) {
        let TestCase { g, iset, oset } = input;
        check_graph(g, iset, oset);
    }
}
