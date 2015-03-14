use capnp;
use capnp::{MessageBuilder, MessageReader, ReaderOptions, MallocMessageBuilder};
use cgmath::{Point, Point3, Vector3, Aabb3};
use std::f32::consts::PI;
use std::io::Cursor;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use time;

use common::block_position::BlockPosition;
use common::communicate::{ClientId, client_to_server, server_to_client};
use common::entity::EntityId;
use common::lod::LODIndex;
use common::socket::SendSocket;

use player::Player;
use server::{Client, Server};
use terrain::terrain;
use update_gaia::{ServerToGaia, LoadReason};

fn center(bounds: &Aabb3<f32>) -> Point3<f32> {
  bounds.min.add_v(&bounds.max.to_vec()).mul_s(1.0 / 2.0)
}

#[inline]
pub fn apply_client_update<UpdateGaia>(
  server: &Server,
  update_gaia: &mut UpdateGaia,
  update: client_to_server::Reader,
) where
  UpdateGaia: FnMut(ServerToGaia),
{
  match update.borrow().which().unwrap() {
    client_to_server::Init(client_url) => {
      let client_url = String::from_str(client_url);
      info!("Sending to {}.", client_url);

      let (to_client_send, to_client_recv) = channel();
      let client_thread = {
        thread::scoped(move || {
          let mut socket = SendSocket::new(client_url.as_slice(), Some(Duration::seconds(30)));
          while let Some(mut msg) = to_client_recv.recv().unwrap() {
            // TODO: Don't do this allocation of every packet.
            let mut buf = Cursor::new(Vec::new());
            {
              let mut buf = capnp::io::WriteOutputStream::new(&mut buf);
              capnp::serialize::write_message(&mut buf, &mut msg).unwrap();
            }
            let buf = buf.into_inner();
            socket.write(buf.as_slice());
          }
        })
      };

      let client_id = server.client_allocator.lock().unwrap().allocate();
      {
        let message =
          capnpc_new!(
            server_to_client::Builder =>
            [init_lease_id => [set_id client_id.0]]
          );
        to_client_send.send(Some(message)).unwrap();
      }

      let client =
        Client {
          sender: to_client_send,
          thread: client_thread,
        };
      server.clients.lock().unwrap().insert(client_id, client);
    },
    client_to_server::Ping(client_id) => {
      let client_id = ClientId(client_id.get_id());
      let message = capnpc_new!(server_to_client::Builder => [set_ping ()]);
      server.clients.lock().unwrap()
        .get(&client_id)
        .unwrap()
        .sender
        .send(Some(message))
        .unwrap();
    },
    client_to_server::AddPlayer(client_id) => {
      let client_id = ClientId(client_id.get_id());
      let mut player =
        Player::new(
          server.id_allocator.lock().unwrap().allocate(),
          &server.owner_allocator,
        );

      let min = Point3::new(0.0, terrain::AMPLITUDE as f32, 4.0);
      let max = min.add_v(&Vector3::new(1.0, 2.0, 1.0));
      let bounds = Aabb3::new(min, max);
      server.physics.lock().unwrap().insert_misc(player.entity_id, bounds.clone());

      player.position = center(&bounds);
      player.rotate_lateral(PI / 2.0);

      let id = player.entity_id;
      let pos = player.position;

      server.players.lock().unwrap().insert(id, player);

      let clients = server.clients.lock().unwrap();
      let client = clients.get(&client_id).unwrap();
      let message =
        capnpc_new!(
          server_to_client::Builder =>
          [init_player_added =>
            [init_id => [set_id id.0]]
            [init_position =>
              [set_x bounds.min.x]
              [set_y bounds.min.y]
              [set_z bounds.min.z]
            ]
          ]
        );
      client.sender.send(Some(message)).unwrap();
    },
    client_to_server::StartJump(player_id) => {
      let player_id = EntityId(player_id.get_id());
      let mut players = server.players.lock().unwrap();
      let player = players.get_mut(&player_id).unwrap();
      if !player.is_jumping {
        player.is_jumping = true;
        // this 0.3 is duplicated in a few places
        player.accel.y = player.accel.y + 0.3;
      }
    },
    client_to_server::StopJump(player_id) => {
      let player_id = EntityId(player_id.get_id());
      let mut players = server.players.lock().unwrap();
      let player = players.get_mut(&player_id).unwrap();
      if player.is_jumping {
        player.is_jumping = false;
        // this 0.3 is duplicated in a few places
        player.accel.y = player.accel.y - 0.3;
      }
    },
    client_to_server::Walk(walk) => {
      let player_id = EntityId(walk.get_player().get_id());
      let da = walk.get_da();
      let da = Vector3::new(da.get_x(), da.get_y(), da.get_z());
      let mut players = server.players.lock().unwrap();
      let mut player = players.get_mut(&player_id).unwrap();
      player.walk(da);
    },
    client_to_server::RotatePlayer(rotate) => {
      let mut player_id = EntityId(rotate.get_player().get_id());
      let rx = rotate.get_rx();
      let ry = rotate.get_ry();
      let mut players = server.players.lock().unwrap();
      let mut player = players.get_mut(&player_id).unwrap();
      player.rotate_lateral(rx);
      player.rotate_vertical(ry);
    },
    client_to_server::RequestBlock(request_block) => {
      let client_id = request_block.get_client().get_id();
      let position = request_block.get_position();
      let position = BlockPosition::new(position.get_x(), position.get_y(), position.get_z());
      let lod = LODIndex(request_block.get_lod_index());
      update_gaia(ServerToGaia::Load(position, lod, LoadReason::ForClient(ClientId(client_id))));
    },
  };
}
