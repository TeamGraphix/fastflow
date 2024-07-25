//! Testing utilities.

use std::sync::LazyLock;

use crate::common::{Graph, Nodes};

pub mod exports {
    pub use std::cmp;

    pub use hashbrown::{HashMap, HashSet};
}

macro_rules! nodes {
    ($($x:expr),*) => {
        $crate::test_utils::exports::HashSet::from_iter([$($x),*].iter().copied())
    };
}

macro_rules! measurements {
    ($($u:literal: $v:expr),*) => {
        $crate::test_utils::exports::HashMap::from_iter([$(($u, ($v).into())),*].iter().copied())
    };
}

macro_rules! static_max {
    ($x:expr) => {
        $x
    };
    ($x:expr, $($y:expr),+) => {
        $x.max(static_max!($($y),+))
    };
}

macro_rules! graph {
    () => {
        vec![]
    };
    ($(($u:expr, $v:expr)),+) => {{
        let n = $crate::test_utils::exports::cmp::max(static_max!($($u),+), static_max!($($v),+)) + 1;
        let mut g = vec![$crate::test_utils::exports::HashSet::new(); n];
        $(
        g[$u].insert($v);
        g[$v].insert($u);
        )+
        g
    }};
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
        g: graph![(0, 1)],
        iset: nodes![0, 1],
        oset: nodes![0, 1],
    }
});

pub static CASE1: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 1 - 2 - 3 - 4
    TestCase {
        g: graph![(0, 1), (1, 2), (2, 3), (3, 4)],
        iset: nodes![0],
        oset: nodes![4],
    }
});

pub static CASE2: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 2 - 4
    //     |
    // 1 - 3 - 5
    TestCase {
        g: graph![(0, 2), (1, 3), (2, 4), (3, 5), (2, 3)],
        iset: nodes![0, 1],
        oset: nodes![4, 5],
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
        g: graph![(0, 3), (0, 5), (1, 3), (1, 4), (1, 5), (2, 4), (2, 5)],
        iset: nodes![0, 1, 2],
        oset: nodes![3, 4, 5],
    }
});

pub static CASE4: LazyLock<TestCase> = LazyLock::new(|| {
    //   0 - 1
    //  /|   |
    // 4 |   |
    //  \|   |
    //   2 - 5 - 3
    TestCase {
        g: graph![(0, 1), (0, 2), (0, 4), (1, 5), (2, 4), (2, 5), (3, 5)],
        iset: nodes![0, 1],
        oset: nodes![4, 5],
    }
});

pub static CASE5: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 2
    //  \ /
    //   X
    //  / \
    // 1 - 3
    TestCase {
        g: graph![(0, 2), (0, 3), (1, 2), (1, 3)],
        iset: nodes![0, 1],
        oset: nodes![2, 3],
    }
});

pub static CASE6: LazyLock<TestCase> = LazyLock::new(|| {
    //     3
    //     |
    //     2
    //     |
    // 0 - 1 - 4
    TestCase {
        g: graph![(0, 1), (1, 2), (1, 4), (2, 3)],
        iset: nodes![0],
        oset: nodes![4],
    }
});

pub static CASE7: LazyLock<TestCase> = LazyLock::new(|| {
    // 1   2   3
    // | /     |
    // 0 - - - 4
    TestCase {
        g: graph![(0, 1), (0, 2), (0, 4), (3, 4)],
        iset: nodes![0],
        oset: nodes![4],
    }
});

pub static CASE8: LazyLock<TestCase> = LazyLock::new(|| {
    // 0 - 1 -- 3
    //    \|   /|
    //     |\ / |
    //     | /\ |
    //     2 -- 4
    TestCase {
        g: graph![(0, 1), (0, 4), (1, 2), (1, 3), (2, 3), (2, 4), (3, 4)],
        iset: nodes![0],
        oset: nodes![3, 4],
    }
});
