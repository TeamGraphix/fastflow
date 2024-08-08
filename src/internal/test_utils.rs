//! Testing utilities.

use std::sync::LazyLock;

use crate::common::{Graph, Nodes};

pub mod exports {
    pub use hashbrown::HashMap;
}

macro_rules! measurements {
    ($($u:literal: $v:expr),*) => {
        $crate::internal::test_utils::exports::HashMap::from_iter([$(($u, ($v).into())),*].iter().copied())
    };
}

/// Creates a undirected graph from edges.
fn graph<const N: usize>(edges: &[(usize, usize); N]) -> Graph {
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
