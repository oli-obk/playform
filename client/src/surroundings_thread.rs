use client::{Client, LOD_THRESHOLDS};
use common::block_position::BlockPosition;
use common::communicate::ClientToServer;
use common::cube_shell::cube_diff;
use common::lod::LODIndex;
use common::stopwatch::TimerSet;
use common::surroundings_loader::{LODChange, SurroundingsLoader};
use std::num;
use std::old_io::timer;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::duration::Duration;
use view_update::ClientToView;
use view_update::ClientToView::*;

pub fn surroundings_thread(
  client: &Client,
  ups_to_view: &Mutex<Sender<ClientToView>>,
  ups_to_server: &Mutex<Sender<ClientToServer>>,
) {
  let timers = TimerSet::new();
  let timers = &timers;

  let mut surroundings_loader = {
    SurroundingsLoader::new(
      client.max_load_distance,
      Box::new(move |last, cur| {
        let mut vec = Vec::new();
        for &r in LOD_THRESHOLDS.iter() {
          vec.push_all(cube_diff(last, cur, r).as_slice());
        }
        vec.push_all(cube_diff(last, cur, client.max_load_distance).as_slice());
        vec
      }),
    )
  };

  loop {
    let block_position = BlockPosition::from_world_position(&client.player_position.lock().unwrap().clone());

    surroundings_loader.update(
      block_position,
      |lod_change| {
        match lod_change {
          LODChange::Load(block_position, distance) => {
            timers.time("request_block", || {
              let lod = lod_index(distance);
              ups_to_server.lock().unwrap().send(ClientToServer::RequestBlock(block_position, lod)).unwrap();
            });
          },
          LODChange::Unload(block_position) => {
            // The block removal code is duplicated elsewhere.

            client.loaded_blocks
              .lock().unwrap()
              .remove(&block_position)
              // If it wasn't loaded, don't unload anything.
              .map(|(block, prev_lod)| {
                timers.time("remove_block", || {
                  for id in block.ids.iter() {
                    ups_to_view.lock().unwrap().send(RemoveTerrain(*id)).unwrap();
                  }

                  ups_to_view.lock().unwrap().send(RemoveBlockData(block_position, prev_lod)).unwrap();
                });
              });
          },
        };
      },
    );

    timer::sleep(Duration::milliseconds(1));
  }

  // TODO: This thread should exit nicely.
  // i.e. reach this point and print `timers`.
}

pub fn lod_index(distance: i32) -> LODIndex {
  assert!(distance >= 0);
  let mut lod = 0;
  while
    lod < LOD_THRESHOLDS.len()
    && LOD_THRESHOLDS[lod] < distance
  {
    lod += 1;
  }
  LODIndex(num::cast(lod).unwrap())
}