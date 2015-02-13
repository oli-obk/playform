use client_update::ServerToClient;
use gaia_thread::gaia_thread;
use gaia_update::ServerToGaia;
use id_allocator::IdAllocator;
use init::world;
use interval_timer::IntervalTimer;
use lod::OwnerId;
use server_update::ClientToServer;
use std::old_io::timer;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::sync::Mutex;
use std::thread::Thread;
use std::time::duration::Duration;
use stopwatch::TimerSet;
use time;
use update::update;

pub const UPDATES_PER_SECOND: u64 = 30;

pub fn server_thread(
  owner_allocator: &mut IdAllocator<OwnerId>,
  ups_from_client: &Receiver<ClientToServer>,
  ups_to_client: &Sender<ServerToClient>,
) {
  let timers = TimerSet::new();

  let id_allocator = Mutex::new(IdAllocator::new());

  let mut world = world::init(&ups_to_client, owner_allocator, &timers);

  let (ups_to_gaia_send, ups_to_gaia_recv) = channel();
  let (ups_from_gaia_send, ups_from_gaia_recv) = channel();
  let _gaia_thread = {
    let terrain = world.terrain_game_loader.terrain.clone();
    Thread::spawn(move || {
      gaia_thread(
        &ups_to_gaia_recv,
        &ups_from_gaia_send,
        &id_allocator,
        terrain,
      );
    })
  };
  let ups_to_gaia = ups_to_gaia_send;
  let ups_from_gaia = ups_from_gaia_recv;

  let mut update_timer;
  {
    let now = time::precise_time_ns();
    let nanoseconds_per_second = 1000000000;
    update_timer = IntervalTimer::new(nanoseconds_per_second / UPDATES_PER_SECOND, now);
  }

  'server_loop:loop {
    'event_loop:loop {
      match ups_from_client.try_recv() {
        Err(TryRecvError::Empty) => break 'event_loop,
        Err(e) => panic!("Error getting world updates: {:?}", e),
        Ok(update) => {
          if !update.apply(&mut world, &ups_to_client, &ups_to_gaia) {
            ups_to_gaia.send(ServerToGaia::Quit).unwrap();
            break 'server_loop;
          }
        },
      }
    };

    'event_loop:loop {
      match ups_from_gaia.try_recv() {
        Err(TryRecvError::Empty) => break 'event_loop,
        Err(e) => panic!("Error getting world updates: {:?}", e),
        Ok(update) => {
          update.apply(&timers, &mut world, &ups_to_client, &ups_to_gaia);
        },
      };
    }

    let updates = update_timer.update(time::precise_time_ns());
    if updates > 0 {
      update(&timers, &mut world, &ups_to_client, &ups_to_gaia);
    }

    timer::sleep(Duration::milliseconds(0));
  }

  debug!("server exiting.");
}
