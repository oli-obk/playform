//! Defines the messages passed between client and server.

use block_position::BlockPosition;
use entity::EntityId;
use nalgebra::{Vec2, Vec3, Pnt3};
use lod::LODIndex;
use terrain_block::TerrainBlock;
use vertex::ColoredVertex;

#[derive(Debug, Clone)]
/// Messages the client sends to the server.
pub enum ClientToServer {
  /// Add a vector the player's acceleration.
  Walk(Vec3<f32>),
  /// Rotate the player by some amount.
  RotatePlayer(Vec2<f32>),
  /// [Try to] start a jump for the player.
  StartJump,
  /// [Try to] stop a jump for the player.
  StopJump,
  /// Ask the server to send a block of terrain.
  RequestBlock(BlockPosition, LODIndex),
  /// Kill the server.
  Quit,
}

/// Messages the server sends to the client.
pub enum ServerToClient {
  /// Update the player's position.
  UpdatePlayer(Pnt3<f32>),

  /// Tell the client to add a new mob with the given mesh.
  AddMob(EntityId, Vec<ColoredVertex>),
  /// Update the client's view of a mob with a given mesh.
  UpdateMob(EntityId, Vec<ColoredVertex>),

  /// The sun as a [0, 1) portion of its cycle.
  UpdateSun(f32),

  /// Provide a block of terrain to a client.
  AddBlock(BlockPosition, TerrainBlock, LODIndex),
}