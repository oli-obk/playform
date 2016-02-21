//! This crate contains client-only components of Playform.

#![allow(unneeded_field_pattern)]
#![deny(missing_docs)]
#![deny(warnings)]

#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]

#![plugin(clippy)]

#![allow(mutex_atomic)]
#![allow(match_ref_pats)]
#![allow(match_same_arms)]

extern crate bincode;
extern crate cgmath;
extern crate common;
extern crate fnv;
extern crate gl;
extern crate isosurface_extraction;
#[macro_use]
extern crate log;
extern crate libc;
extern crate num;
extern crate sdl2;
extern crate sdl2_sys;
extern crate stopwatch;
extern crate rustc_serialize;
extern crate test;
extern crate thread_scoped;
extern crate time;
extern crate voxel_data;
extern crate yaglw;

mod block_position;
mod camera;
mod client;
mod edge;
mod hud;
mod light;
mod load_terrain;
mod loaded_edges;
mod lod;
mod mob_buffers;
mod player_buffers;
mod process_event;
mod record_book;
mod render;
mod run;
mod server;
mod server_update;
mod shaders;
mod terrain_buffers;
mod terrain_mesh;
mod update_thread;
mod vertex;
mod voxel;
mod view;
mod view_thread;
mod view_update;

pub use run::run;
