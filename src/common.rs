use num_traits::{NumOps, Zero, One};
use std::fmt::{Display, Debug};
use std::hash::Hash;

pub type HeaderId = usize;
pub type NodeId = usize;
pub type Level = usize;

pub type HashMap<T,U> = std::collections::HashMap<T,U>;
pub type HashSet<T> = std::collections::HashSet<T>;
// pub type HashMap<T,U> = hashbrown::HashMap<T,U>;
// pub type HashSet<T> = hashbrown::HashSet<T>;

pub trait TerminalBinaryValue:
    Copy
    + Clone
    + PartialEq
    + Eq
    + Hash
    + Display
    + Debug
{
    fn high() -> Self;
    fn low() -> Self;
}

impl TerminalBinaryValue for bool {
    fn high() -> Self { true }
    fn low() -> Self { false }
}

impl TerminalBinaryValue for u8 {
    fn high() -> Self { 1 }
    fn low() -> Self { 0 }
}

pub trait TerminalNumberValue:
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

impl TerminalNumberValue for u32 {}
impl TerminalNumberValue for u64 {}
impl TerminalNumberValue for i32 {}
impl TerminalNumberValue for i64 {}

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

