pub mod bounds {
  use block_position;
  use lod;
  use terrain_mesh;

  pub use common::voxel::bounds::*;

  /// Find the LOD that this voxel should be loaded at, and return the voxel(s) that represent the same space at that LOD.
  pub fn correct_lod(bounds: &T, player_position: &block_position::T) -> Vec<T> {
    let block_position = block_position::containing(bounds);
    let lod: lod::T = block_position.desired_lod(&player_position);
    let lg_size = terrain_mesh::LG_SAMPLE_SIZE[lod.0 as usize];

    let mut cpy = bounds.clone();
    cpy.lg_size = lg_size;

    if lg_size > bounds.lg_size {
      let lg_ratio = lg_size - bounds.lg_size;
      cpy.x = cpy.x >> lg_ratio;
      cpy.y = cpy.y >> lg_ratio;
      cpy.z = cpy.z >> lg_ratio;
      vec!(cpy)
    } else if lg_size < bounds.lg_size {
      let lg_ratio = bounds.lg_size - lg_size;
      cpy.x = cpy.x << lg_ratio;
      cpy.y = cpy.y << lg_ratio;
      cpy.z = cpy.z << lg_ratio;
      (0 .. (1 << lg_ratio)).flat_map(|dx| {
      (0 .. (1 << lg_ratio)).flat_map(move |dy| {
      (0 .. (1 << lg_ratio)).map(move |dz| {
        let mut cpy = cpy.clone();
        cpy.x = cpy.x + dx;
        cpy.y = cpy.y + dy;
        cpy.z = cpy.z + dz;
        cpy
      })})})
      .collect()
    } else {
      vec!(cpy)
    }
  }

  pub mod set {
    use fnv::FnvHasher;
    use std;

    pub type T = std::collections::HashSet<super::T, std::hash::BuildHasherDefault<FnvHasher>>;

    pub fn new() -> T {
      std::collections::HashSet::with_hasher(Default::default())
    }
  }
}

pub use common::voxel::T;
pub use common::voxel::tree;
pub use common::voxel::Material;
pub use voxel_data::impls::surface_vertex::T::*;
pub use voxel_data::impls::surface_vertex::{of_field, unwrap};
