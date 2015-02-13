//! The entry point.
#![crate_type = "bin"]
#![deny(missing_docs)]
#![deny(warnings)]

#![feature(core)]
#![feature(collections)]
#![feature(hash)]
#![feature(io)]
#![feature(path)]
#![feature(slicing_syntax)]
#![feature(std_misc)]
#![feature(test)]
#![feature(unboxed_closures)]
#![feature(unsafe_destructor)]

extern crate gl;
#[macro_use]
extern crate log;
extern crate nalgebra;
extern crate ncollide_entities;
extern crate ncollide_queries;
extern crate noise;
extern crate opencl;
extern crate rand;
extern crate sdl2;
extern crate "sdl2-sys" as sdl2_sys;
extern crate test;
extern crate time;
extern crate yaglw;

mod camera;
mod client;
mod client_thread;
mod client_update;
mod color;
mod common;
mod cube_shell;
mod fontloader;
mod gaia_thread;
mod gaia_update;
mod id_allocator;
mod in_progress_terrain;
mod init;
mod interval_timer;
mod light;
mod loader;
mod lod;
mod logger;
mod main;
mod mob;
mod octree;
mod opencl_context;
mod physics;
mod player;
mod process_event;
mod range_abs;
mod render;
mod server;
mod server_thread;
mod server_update;
mod shaders;
mod stopwatch;
mod sun;
mod surroundings_iter;
mod surroundings_loader;
mod terrain;
mod ttf;
mod update;
mod vertex;
mod view;
mod view_thread;
mod view_update;

#[allow(dead_code)]
fn main() {
  return main::main();
}
