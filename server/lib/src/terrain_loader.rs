use cgmath::{Aabb3};
use std::collections::HashMap;
use std::sync::Mutex;
use stopwatch;
use time;

use common::entity_id;
use common::id_allocator;
use common::voxel;

use in_progress_terrain;
use lod;
use physics::Physics;
use terrain;
use update_gaia;
use update_gaia::LoadReason;

// TODO: Consider factoring this logic such that what to load is separated from how it's loaded.

/// Load and unload terrain::TerrainBlocks from the game.
/// Each terrain::TerrainBlock can be owned by a set of owners, each of which can independently request LODs.
/// The maximum lod::T requested is the one that is actually loaded.
pub struct T {
  pub terrain: terrain::T,
  pub in_progress_terrain: Mutex<in_progress_terrain::T>,
  pub lod_map: Mutex<lod::Map>,
  pub loaded: Mutex<HashMap<voxel::bounds::T, Vec<entity_id::T>>>,
}

impl T {
  pub fn new() -> T {
    T {
      terrain: terrain::T::new(terrain::Seed::new(0)),
      in_progress_terrain: Mutex::new(in_progress_terrain::T::new()),
      lod_map: Mutex::new(lod::Map::new()),
      loaded: Mutex::new(HashMap::new()),
    }
  }

  // TODO: Avoid the double-lookup when unload and load the same index.

  pub fn load<LoadBlock>(
    &self,
    id_allocator: &Mutex<id_allocator::T<entity_id::T>>,
    physics: &Mutex<Physics>,
    block_position: &voxel::bounds::T,
    new_lod: lod::T,
    owner: lod::OwnerId,
    load_block: &mut LoadBlock,
  ) where LoadBlock: FnMut(update_gaia::Message)
  {
    let prev_lod;
    let max_lod_changed: bool;
    let mut lod_map = self.lod_map.lock().unwrap();
    let mut in_progress_terrain = self.in_progress_terrain.lock().unwrap();
    match lod_map.get(block_position, owner) {
      Some((Some(prev), lods)) => {
        prev_lod = Some(prev);
        if new_lod == prev {
          return;
        }

        if new_lod < prev {
          max_lod_changed = lods.iter().filter(|&&(_, l)| l >= prev).count() < 2;
        } else {
          max_lod_changed = lods.iter().filter(|&&(_, l)| l >= new_lod).count() == 0;
        }
      },
      Some((None, lods)) => {
        max_lod_changed = lods.iter().filter(|&&(_, l)| l >= new_lod).count() == 0;
        prev_lod = None;
      },
      None => {
        max_lod_changed = true;
        prev_lod = None;
      },
    }

    if !max_lod_changed {
      // Maximum lod::T is unchanged.
      let (_, change) = lod_map.insert(*block_position, new_lod, owner);
      assert!(change.is_none());
      return;
    }

    match new_lod {
      lod::Placeholder => {
        let (_, change) = lod_map.insert(*block_position, new_lod, owner);
        let change = change.unwrap();
        assert!(change.loaded == None);
        assert!(prev_lod == None);
        assert!(change.desired == Some(lod::Placeholder));
        in_progress_terrain.insert(id_allocator, physics, block_position);
      },
      lod::Full => {
        let mut generate_block = || {
          debug!("{:?} requested from gaia", block_position);
          load_block(
            update_gaia::Message::Load(time::precise_time_ns(), vec!(*block_position), LoadReason::Local(owner))
          );
        };
        generate_block();
      },
    };
  }

  pub fn insert_block(
    block: &LoadedTerrain,
    position: &voxel::bounds::T,
    owner: lod::OwnerId,
    physics: &Mutex<Physics>,
    lod_map: &mut lod::Map,
    in_progress_terrain: &mut in_progress_terrain::T,
    loaded: &mut HashMap<voxel::bounds::T, Vec<entity_id::T>>,
  ) {
    let lod = lod::Full;
    let (_, change) = lod_map.insert(*position, lod, owner);
    // TODO: This should be an unwrap, but the preconditions of another TODO aren't
    // satisfied in src/update_gaia.rs.
    // (i.e. blocks sometimes get here when they're stale).
    let change = match change {
      None => return,
      Some(change) => change,
    };
    assert!(change.desired == Some(lod));
    change.loaded.map(|loaded_lod|
      match loaded_lod {
        lod::Placeholder => {
          in_progress_terrain.remove(physics, position);
        }
        lod::Full => {
          stopwatch::time("terrain_loader.load.unload", || {
            let ids = loaded.get(position).unwrap();
            let mut physics = physics.lock().unwrap();
            for id in ids {
              physics.remove_terrain(*id);
            }
          });
        },
      }
    );

    stopwatch::time("terrain_loader.load.physics", || {
      let mut physics = physics.lock().unwrap();
      for &(ref id, ref bounds) in &block.bounds {
        physics.insert_terrain(*id, bounds);
      }
    });
  }

  pub fn unload(
    &self,
    physics: &Mutex<Physics>,
    block_position: &voxel::bounds::T,
    owner: lod::OwnerId,
  ) {
    let (_, mlod_change) =
      self.lod_map.lock().unwrap().remove(*block_position, owner);

    let lod_change;
    match mlod_change {
      None => {
        return;
      },
      Some(c) => lod_change = c,
    }

    lod_change.loaded.map(|loaded_lod| {
      match loaded_lod {
        lod::Placeholder => {
          self.in_progress_terrain.lock().unwrap().remove(physics, block_position);
        }
        lod::Full => {
          stopwatch::time("terrain_loader.unload", || {
            match self.loaded.lock().unwrap().get(block_position) {
              None => {
                // Unloaded before the load request completed.
              },
              Some(ids) => {
                let mut physics = physics.lock().unwrap();
                for id in ids {
                  physics.remove_terrain(*id);
                }
              },
            }
          });
        },
      }
    });
  }
}

pub struct LoadedTerrain {
  pub bounds: Vec<(entity_id::T, Aabb3<f32>)>,
}
