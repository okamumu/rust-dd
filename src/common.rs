use num_traits::{NumOps, One, Zero};
use std::fmt::Display;
use std::hash::{BuildHasherDefault, Hash};
use wyhash::WyHash;

pub type HeaderId = usize;
pub type NodeId = usize;
pub type Level = usize;
pub type OperationId = usize;

// pub type HashMap<T,U> = std::collections::HashMap<T,U>;
// pub type HashSet<T> = std::collections::HashSet<T>;
pub type HashMap<T, U> = std::collections::HashMap<T, U, BuildHasherDefault<WyHash>>;
pub type HashSet<T> = std::collections::HashSet<T, BuildHasherDefault<WyHash>>;
// pub type HashMap<T,U> = hashbrown::HashMap<T,U>;
// pub type HashSet<T> = hashbrown::HashSet<T>;

pub trait TerminalNumberValue:
    Copy + Clone + PartialEq + Eq + Hash + NumOps + Display + Ord + Zero + One
{
}

impl TerminalNumberValue for u32 {}
impl TerminalNumberValue for u64 {}
impl TerminalNumberValue for i32 {}
impl TerminalNumberValue for i64 {}

pub trait EdgeValue: Copy + Clone + PartialEq + Eq + Hash + NumOps + Display + Ord + Zero {}

impl EdgeValue for i32 {}
impl EdgeValue for i64 {}
