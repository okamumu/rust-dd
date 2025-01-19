use wyhash::WyHash;
use std::hash::BuildHasherDefault;

pub type HeaderId = usize;
pub type NodeId = usize;
pub type Level = usize;
pub type OperationId = usize;

// pub type HashMap<T,U> = std::collections::HashMap<T,U>;
// pub type HashSet<T> = std::collections::HashSet<T>;
// pub type HashMap<T,U> = hashbrown::HashMap<T,U>;
// pub type HashSet<T> = hashbrown::HashSet<T>;

pub type BddHashMap<T, U> = std::collections::HashMap<T, U, BuildHasherDefault<WyHash>>;
pub type BddHashSet<T> = std::collections::HashSet<T, BuildHasherDefault<WyHash>>;
