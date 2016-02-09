use cgmath::{Aabb, Point3, Vector3, Point, Vector};

use block_position;
use lod;
use terrain_mesh;
use voxel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction { X, Y, Z }

impl Direction {
  pub fn to_vec(self) -> Vector3<i32> {
    match self {
      Direction::X => Vector3::new(1, 0, 0),
      Direction::Y => Vector3::new(0, 1, 0),
      Direction::Z => Vector3::new(0, 0, 1),
    }
  }

  pub fn perpendicular(self) -> (Direction, Direction) {
    match self {
      Direction::X => (Direction::Y, Direction::Z),
      Direction::Y => (Direction::Z, Direction::X),
      Direction::Z => (Direction::X, Direction::Y),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct T {
  pub low_corner: Point3<i32>,
  pub lg_size: i16,
  pub direction: Direction,
}

impl T {
  pub fn neighbors(&self) -> [voxel::bounds::T; 4] {
    let (v1, v2): (Direction, Direction) = self.direction.perpendicular();
    let (v1, v2) = (-v1.to_vec(), -v2.to_vec());
    let make_bounds = |p: Point3<i32>| voxel::bounds::new(p.x, p.y, p.z, self.lg_size);
    [
      make_bounds(self.low_corner),
      make_bounds(self.low_corner.add_v(&v1)),
      make_bounds(self.low_corner.add_v(&v1).add_v(&v2)),
      make_bounds(self.low_corner.add_v(&v2)),
    ]
  }

  /// Find the LOD that this edge should be loaded at, and return the edge(s) that represent the same space at that LOD.
  pub fn correct_lod(&self, player_position: &block_position::T) -> Vec<T> {
    let lod: lod::T =
      self.neighbors().iter()
      .map(|bounds| block_position::containing(&bounds).desired_lod(player_position))
      .min()
      .unwrap();
    let lg_size = terrain_mesh::LG_SAMPLE_SIZE[lod.0 as usize];

    let mut cpy = self.clone();
    cpy.lg_size = lg_size;

    if lg_size > self.lg_size {
      let lg_ratio = lg_size - self.lg_size;
      cpy.low_corner.x = cpy.low_corner.x >> lg_ratio;
      cpy.low_corner.y = cpy.low_corner.y >> lg_ratio;
      cpy.low_corner.z = cpy.low_corner.z >> lg_ratio;
      vec!(cpy)
    } else if lg_size < self.lg_size {
      let lg_ratio = self.lg_size - lg_size;
      cpy.low_corner.x = cpy.low_corner.x << lg_ratio;
      cpy.low_corner.y = cpy.low_corner.y << lg_ratio;
      cpy.low_corner.z = cpy.low_corner.z << lg_ratio;

      (0 .. (1 << lg_ratio)).map(|i| {
        let mut cpy = cpy.clone();
        cpy.low_corner.add_self_v(&self.direction.to_vec().mul_s(i));
        cpy
      })
      .collect()
    } else {
      vec!(cpy)
    }
  }
}

pub mod set {
  use fnv::FnvHasher;
  use std;

  pub type T = std::collections::HashSet<super::T, std::hash::BuildHasherDefault<FnvHasher>>;

  #[allow(dead_code)]
  pub fn new() -> T {
    std::collections::HashSet::with_hasher(Default::default())
  }
}

pub mod map {
  use fnv::FnvHasher;
  use std;

  pub type T<V> = std::collections::HashMap<super::T, V, std::hash::BuildHasherDefault<FnvHasher>>;

  pub fn new<V>() -> T<V> {
    std::collections::HashMap::with_hasher(Default::default())
  }
}
