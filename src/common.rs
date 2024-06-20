use std::collections::{HashMap, HashSet};

pub type Flow = HashMap<usize, usize>;
pub type GFlow = HashMap<usize, HashSet<usize>>;
pub type Layer = Vec<usize>;
