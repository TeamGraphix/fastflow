//! Testing utilities.

use std::sync::OnceLock;

use crate::common::{Graph, Nodes};

#[macro_export]
macro_rules! nodeset {
    ($($x:expr),*) => {
        ::hashbrown::HashSet::from_iter([$($x),*].iter().copied())
    };
}
macro_rules! nodeset {
    ($($x:expr),*) => {
        ::hashbrown::HashSet::from_iter([$($x),*].iter().copied())
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
    ($(($u:literal, $v:literal)),+) => {{
        let n = static_max!($($u),+).max(static_max!($($v),+)) + 1;
        let mut g = vec![::hashbrown::HashSet::new(); n];
        $(
            g[$u].insert($v);
            g[$v].insert($u);
        )*
        g
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    pub g: Graph,
    pub iset: Nodes,
    pub oset: Nodes,
}

pub fn case0() -> TestCase {
    // 0 - 1
    TestCase {
        g: graph![(0, 1)],
        iset: nodeset![0, 1],
        oset: nodeset![0, 1],
    }
}

pub fn case1() -> TestCase {
    // 0 - 1 - 2 - 3 - 4
    TestCase {
        g: graph![(0, 1), (1, 2), (2, 3), (3, 4)],
        iset: nodeset![0],
        oset: nodeset![4],
    }
}

pub fn case2() -> TestCase {
    // 0 - 2 - 4
    //     |
    // 1 - 3 - 5
    TestCase {
        g: graph![(0, 2), (1, 3), (2, 4), (3, 5), (2, 3)],
        iset: nodeset![0, 1],
        oset: nodeset![4, 5],
    }
}

pub fn case3() -> TestCase {
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
        iset: nodeset![0, 1, 2],
        oset: nodeset![3, 4, 5],
    }
}

pub fn case4() -> TestCase {
    //   0 - 1
    //  /|   |
    // 4 |   |
    //  \|   |
    //   2 - 5 - 3
    TestCase {
        g: graph![(0, 1), (0, 2), (0, 4), (1, 5), (2, 4), (2, 5), (3, 5)],
        iset: nodeset![0, 1],
        oset: nodeset![4, 5],
    }
}

pub fn case5() -> TestCase {
    // 0 - 2
    //  \ /
    //   X
    //  / \
    // 1 - 3
    TestCase {
        g: graph![(0, 2), (0, 3), (1, 2), (1, 3)],
        iset: nodeset![0, 1],
        oset: nodeset![2, 3],
    }
}

pub fn case6() -> TestCase {
    //     3
    //     |
    //     2
    //     |
    // 0 - 1 - 4
    TestCase {
        g: graph![(0, 1), (1, 2), (1, 4), (2, 3)],
        iset: nodeset![0],
        oset: nodeset![4],
    }
}

pub fn case7() -> TestCase {
    // 1   2   3
    // | /     |
    // 0 - - - 4
    TestCase {
        g: graph![(0, 1), (0, 2), (0, 4), (3, 4)],
        iset: nodeset![0],
        oset: nodeset![4],
    }
}

pub fn case8() -> TestCase {
    // 0 - 1 -- 3
    //    \|   /|
    //     |\ / |
    //     | /\ |
    //     2 -- 4
    TestCase {
        g: graph![(0, 1), (0, 4), (1, 2), (1, 3), (2, 3), (2, 4), (3, 4)],
        iset: nodeset![0],
        oset: nodeset![3, 4],
    }
}

pub static CASE0: OnceLock<TestCase> = OnceLock::new();
pub static CASE1: OnceLock<TestCase> = OnceLock::new();
pub static CASE2: OnceLock<TestCase> = OnceLock::new();
pub static CASE3: OnceLock<TestCase> = OnceLock::new();
pub static CASE4: OnceLock<TestCase> = OnceLock::new();
pub static CASE5: OnceLock<TestCase> = OnceLock::new();
pub static CASE6: OnceLock<TestCase> = OnceLock::new();
pub static CASE7: OnceLock<TestCase> = OnceLock::new();
pub static CASE8: OnceLock<TestCase> = OnceLock::new();
