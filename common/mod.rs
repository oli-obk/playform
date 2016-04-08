//! Data structures and functions shared between server and client.

#![deny(missing_docs)]
#![deny(warnings)]

#![allow(items_after_statements)]
#![allow(enum_variant_names)]
#![allow(doc_markdown)]

#![feature(box_syntax)]
#![feature(fn_traits)]
#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]

#![plugin(clippy)]
#![allow(type_complexity)]

extern crate bincode;
extern crate cgmath;
extern crate isosurface_extraction;
#[macro_use]
extern crate log;
extern crate nanomsg;
extern crate num;
extern crate rustc_serialize;
extern crate stopwatch;
extern crate test;
extern crate time;
extern crate voxel_data;

pub mod bin_encode;
pub mod closure_series;
pub mod color;
pub mod cube_shell;
pub mod entity_id;
pub mod id_allocator;
pub mod interval_timer;
pub mod protocol;
pub mod range_abs;
pub mod socket;
pub mod surroundings_loader;
pub mod voxel;

pub use self::bin_encode::BinEncode;
