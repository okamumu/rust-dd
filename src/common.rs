use std::hash::Hash;
use num_traits::{NumOps, Zero, One};
use std::fmt::Display;

pub type HeaderId = usize;
pub type NodeId = usize;
pub type Level = usize;

pub type HashMap<T,U> = std::collections::HashMap<T,U>;
pub type HashSet<T> = std::collections::HashSet<T>;
// pub type HashMap<T,U> = hashbrown::HashMap<T,U>;
// pub type HashSet<T> = hashbrown::HashSet<T>;

pub trait TerminalValue:
    Copy
    + Clone
    + PartialEq
    + Eq
    + Hash
    + NumOps
    + Display
    + Ord
    + Zero
    + One
    {}

impl TerminalValue for u32 {}
impl TerminalValue for u64 {}
impl TerminalValue for i32 {}
impl TerminalValue for i64 {}

pub trait EdgeValue:
    Copy
    + Clone
    + PartialEq
    + Eq
    + Hash
    + NumOps
    + Display
    + Ord
    + Zero
    {}

impl EdgeValue for i32 {}
impl EdgeValue for i64 {}
