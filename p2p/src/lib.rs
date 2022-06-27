//! P2P communications library for the DAGchain

#![forbid(
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types
)]
#![warn(clippy::all)]

/// P2p-related errors
pub mod error;
/// Functionality of a node on the network
pub mod node;
