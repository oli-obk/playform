use std::sync::mpsc::{channel, Receiver};
use std::thread;

use common::communicate::{ClientToServer, ServerToClient};

use client_send_thread::client_send_thread;
use server::Server;
use update_gaia::{ServerToGaia, LoadReason};

pub fn client_recv_thread<UpdateGaia>(
  server: &Server,
  ups_from_client: &Receiver<ClientToServer>,
  update_gaia: &mut UpdateGaia,
) where UpdateGaia: FnMut(ServerToGaia)
{
  // TODO: Proper exit semantics for this and other threads.
  loop {
    let update = ups_from_client.recv().unwrap();
    match update {
      ClientToServer::Init(client_url) => {
        info!("Sending to {}.", client_url);

        let (to_client_send, to_client_recv) = channel();
        let client_thread = {
          thread::scoped(move || {
            client_send_thread(
              client_url,
              &mut move || { to_client_recv.recv().unwrap() },
            );
          })
        };
        let player_position = server.player.lock().unwrap().position;

        to_client_send.send(
          Some(ServerToClient::UpdatePlayer(player_position))
        ).unwrap();
        server.inform_client(
          &mut |msg| to_client_send.send(Some(msg)).unwrap()
        );

        *server.to_client.lock().unwrap() = Some((to_client_send, client_thread));
      },
      ClientToServer::StartJump => {
        let mut player = server.player.lock().unwrap();
        if !player.is_jumping {
          player.is_jumping = true;
          // this 0.3 is duplicated in a few places
          player.accel.y = player.accel.y + 0.3;
        }
      },
      ClientToServer::StopJump => {
        let mut player = server.player.lock().unwrap();
        if player.is_jumping {
          player.is_jumping = false;
          // this 0.3 is duplicated in a few places
          player.accel.y = player.accel.y - 0.3;
        }
      },
      ClientToServer::Walk(v) => {
        let mut player = server.player.lock().unwrap();
        player.walk(v);
      },
      ClientToServer::RotatePlayer(v) => {
        let mut player = server.player.lock().unwrap();
        player.rotate_lateral(v.x);
        player.rotate_vertical(v.y);
      },
      ClientToServer::RequestBlock(position, lod) => {
        update_gaia(ServerToGaia::Load(position, lod, LoadReason::ForClient));
      },
    };
  }
}