//! This crate contains the terrain data structures and generation.

#![allow(let_and_return)]
#![allow(match_ref_pats)]
#![allow(similar_names)]
#![allow(type_complexity)]
#![allow(unneeded_field_pattern)]
#![allow(derive_hash_xor_eq)]

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(main)]
#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]

#![plugin(clippy)]

extern crate bincode;
extern crate cgmath;
extern crate common;
#[macro_use]
extern crate log;
extern crate noise;
extern crate num;
extern crate rand;
extern crate stopwatch;
extern crate rustc_serialize;
extern crate test;
extern crate time;
extern crate voxel_data;

mod cache_mosaic;

pub mod biome;
pub mod tree;

pub use noise::Seed;

use cgmath::Aabb;
use std::sync::Mutex;

use common::voxel;

#[derive(RustcEncodable)]
struct Encodable<'a> {
  pub seed: Vec<u8>,
  pub voxels: std::sync::MutexGuard<'a, voxel::tree::T>,
}

impl<'a> Encodable<'a> {
  pub fn of(t: &'a T) -> Encodable<'a> {
    let len = std::mem::size_of::<Seed>();
    let seed =
      unsafe {
        Vec::from_raw_parts(std::mem::transmute(&t.seed), len, len)
      };
    Encodable {
      seed: seed,
      voxels: t.voxels.lock().unwrap(),
    }
  }
}

#[derive(RustcDecodable)]
struct Decodable {
  pub seed: Vec<u8>,
  pub voxels: voxel::tree::T,
}

impl Decodable {
  pub fn reconstruct(self) -> T {
    let len = std::mem::size_of::<Seed>();
    assert!(len == self.seed.len());
    let seed1: Seed = unsafe { std::mem::transmute_copy(&*self.seed.as_ptr()) };
    let seed2: Box<Seed> = unsafe { std::mem::transmute(self.seed.as_ptr()) };
    T {
      seed: seed1,
      mosaic: Mutex::new(mosaic(*seed2)),
      voxels: Mutex::new(self.voxels),
    }
  }
}

/// This struct contains and lazily generates the world's terrain.
#[allow(missing_docs)]
pub struct T {
  seed: Seed,
  pub mosaic: Mutex<cache_mosaic::T<voxel::Material>>,
  pub voxels: Mutex<voxel::tree::T>,
}

impl rustc_serialize::Encodable for T {
  fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    Encodable::of(self).encode(s)
  }
}

impl rustc_serialize::Decodable for T {
  fn decode<S: rustc_serialize::Decoder>(s: &mut S) -> Result<Self, S::Error> {
    <Decodable as rustc_serialize::Decodable>::decode(s).map(|d| d.reconstruct())
  }
}

fn mosaic(seed: Seed) -> cache_mosaic::T<voxel::Material> {
  cache_mosaic::new(Box::new(biome::demo::new(seed)))
}

impl T {
  #[allow(missing_docs)]
  pub fn new(terrain_seed: Seed) -> T {
    let seed: Seed = unsafe { std::mem::transmute_copy(&terrain_seed) };
    T {
      seed: seed,
      mosaic: Mutex::new(mosaic(terrain_seed)),
      voxels: Mutex::new(voxel::tree::new()),
    }
  }

  /// Load the block of terrain at a given position.
  // TODO: Allow this to be performed in such a way that self is only briefly locked.
  pub fn load<F>(
    &self,
    bounds: &voxel::bounds::T,
    mut f: F
  ) where
    F: FnMut(&voxel::T)
  {
    let mut voxels = self.voxels.lock().unwrap();
    let branches = voxels.get_mut_or_create(bounds);
    let branches = branches.force_branches();
    match branches.data {
      None => {
        let mut mosaic = self.mosaic.lock().unwrap();
        let voxel = voxel::unwrap(voxel::of_field(&mut *mosaic, bounds));
        f(&voxel);
        branches.data = Some(voxel);
      },
      Some(ref data) => {
        f(data);
      },
    }
  }

  /// Apply a voxel brush to the terrain.
  pub fn brush<VoxelChanged, Mosaic>(
    &self,
    brush: &mut voxel::brush::T<Mosaic>,
    mut voxel_changed: VoxelChanged,
  ) where
    VoxelChanged: FnMut(&voxel::T, &voxel::bounds::T),
    Mosaic: voxel::mosaic::T<voxel::Material>,
  {
    let mut voxels = self.voxels.lock().unwrap();
    voxels.brush(
      brush,
      // TODO: Put a max size on this
      &mut |bounds| {
        if bounds.lg_size > 3 {
          None
        } else {
          let mut mosaic = self.mosaic.lock().unwrap();
          Some(voxel::unwrap(voxel::of_field(&mut *mosaic, bounds)))
        }
      },
      &mut voxel_changed,
    );
  }
}
