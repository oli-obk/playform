use cgmath::{Aabb3, Point, Point3, Vector, Vector3};
use std::f32;
use std::f32::consts::PI;
use stopwatch;
use time;

use common::color::{Color3, Color4};
use common::protocol;
use common::voxel;

use client;
use light;
use vertex::ColoredVertex;
use view_update;

pub const TRIANGLES_PER_BOX: u32 = 12;
pub const VERTICES_PER_TRIANGLE: u32 = 3;
pub const TRIANGLE_VERTICES_PER_BOX: u32 = TRIANGLES_PER_BOX * VERTICES_PER_TRIANGLE;

fn center(bounds: &Aabb3<f32>) -> Point3<f32> {
  bounds.min.add_v(&bounds.max.to_vec()).mul_s(0.5)
}

pub fn apply_server_update<UpdateView, UpdateServer, EnqueueBlockUpdates>(
  client: &client::T,
  update_view: &mut UpdateView,
  update_server: &mut UpdateServer,
  enqueue_block_updates: &mut EnqueueBlockUpdates,
  update: protocol::ServerToClient,
) where
  UpdateView: FnMut(view_update::T),
  UpdateServer: FnMut(protocol::ClientToServer),
  EnqueueBlockUpdates: FnMut(Option<u64>, Vec<(voxel::bounds::T, voxel::T)>, protocol::VoxelReason),
{
  stopwatch::time("apply_server_update", move || {
    match update {
      protocol::ServerToClient::LeaseId(_) => {
        warn!("Client ID has already been leased.");
      },
      protocol::ServerToClient::Ping => {
        update_server(protocol::ClientToServer::Ping(client.id));
      },
      protocol::ServerToClient::PlayerAdded(id, _) => {
        warn!("Unexpected PlayerAdded event: {:?}.", id);
      },
      protocol::ServerToClient::UpdatePlayer(player_id, bounds) => {
        let mesh = to_triangles(&bounds, &Color4::of_rgba(0.0, 0.0, 1.0, 1.0));
        update_view(view_update::UpdatePlayer(player_id, mesh));

        // We "lock" the client to client.player_id, so for updates to that player only,
        // there is more client-specific logic.
        if player_id != client.player_id {
          return
        }

        let position = center(&bounds);

        *client.player_position.lock().unwrap() = position;
        update_view(view_update::MoveCamera(position));
      },
      protocol::ServerToClient::UpdateMob(id, bounds) => {
        let mesh = to_triangles(&bounds, &Color4::of_rgba(1.0, 0.0, 0.0, 1.0));
        update_view(view_update::UpdateMob(id, mesh));
      },
      protocol::ServerToClient::UpdateSun(fraction) => {
        // Convert to radians.
        let angle = fraction * 2.0 * PI;
        let (s, c) = angle.sin_cos();

        let sun_color =
          Color3::of_rgb(
            c.abs(),
            (s + 1.0) / 2.0,
            (s * 0.75 + 0.25).abs(),
          );

        update_view(view_update::SetSun(
          light::Sun {
            direction: Vector3::new(c, s, 0.0),
            intensity: sun_color,
          }
        ));

        let ambient_light = f32::max(0.4, s / 2.0);

        update_view(view_update::SetAmbientLight(
          Color3::of_rgb(
            sun_color.r * ambient_light,
            sun_color.g * ambient_light,
            sun_color.b * ambient_light,
          ),
        ));

        update_view(view_update::SetClearColor(sun_color));
      },
      protocol::ServerToClient::Voxels(request_time, voxels, reason) => {
        match request_time {
          None => {},
          Some(request_time) => debug!("Receiving a voxel request after {}ns", time::precise_time_ns() - request_time),
        }

        enqueue_block_updates(request_time, voxels, reason);
      },
    }
  })
}

fn to_triangles(
  bounds: &Aabb3<f32>,
  c: &Color4<f32>,
) -> [ColoredVertex; TRIANGLE_VERTICES_PER_BOX as usize] {
  let (x1, y1, z1) = (bounds.min.x, bounds.min.y, bounds.min.z);
  let (x2, y2, z2) = (bounds.max.x, bounds.max.y, bounds.max.z);

  let vtx = |x, y, z| {
    ColoredVertex {
      position: Point3::new(x, y, z),
      color: *c,
    }
  };

  // Remember: x increases to the right, y increases up, and z becomes more
  // negative as depth from the viewer increases.
  [
    // front
    vtx(x1, y1, z2), vtx(x2, y2, z2), vtx(x1, y2, z2),
    vtx(x1, y1, z2), vtx(x2, y1, z2), vtx(x2, y2, z2),
    // left
    vtx(x1, y1, z1), vtx(x1, y2, z2), vtx(x1, y2, z1),
    vtx(x1, y1, z1), vtx(x1, y1, z2), vtx(x1, y2, z2),
    // top
    vtx(x1, y2, z1), vtx(x2, y2, z2), vtx(x2, y2, z1),
    vtx(x1, y2, z1), vtx(x1, y2, z2), vtx(x2, y2, z2),
    // back
    vtx(x1, y1, z1), vtx(x2, y2, z1), vtx(x2, y1, z1),
    vtx(x1, y1, z1), vtx(x1, y2, z1), vtx(x2, y2, z1),
    // right
    vtx(x2, y1, z1), vtx(x2, y2, z2), vtx(x2, y1, z2),
    vtx(x2, y1, z1), vtx(x2, y2, z1), vtx(x2, y2, z2),
    // bottom
    vtx(x1, y1, z1), vtx(x2, y1, z2), vtx(x1, y1, z2),
    vtx(x1, y1, z1), vtx(x2, y1, z1), vtx(x2, y1, z2),
  ]
}
