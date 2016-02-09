//! Structs for keeping track of terrain level of detail.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A strongly-typed index into various LOD-indexed arrays.
/// 0 is the highest LOD.
pub struct T(pub u32);

// TODO: Reverse the direction of PartialOrd and Ord.
