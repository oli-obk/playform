@0xc9c559f5cdba0167;

struct Vector3 {
  x @0: Float32;
  y @1: Float32;
  z @2: Float32;
}

struct Color3 {
  r @0: Float32;
  g @1: Float32;
  b @2: Float32;
}

struct ClientId {
  id @0: UInt32;
}

struct EntityId {
  id @0: UInt32;
}

struct BlockPosition {
  x @0: Int32;
  y @1: Int32;
  z @2: Int32;
}

struct Walk {
  player @0: EntityId;
  da @1: Vector3;
}

struct ClientToServer {
  union {
    init @0: Text;
    ping @1: ClientId;
    addPlayer @2: ClientId;
    walk @3: Walk;
    rotatePlayer @4: RotatePlayer;
    startJump @5: EntityId;
    stopJump @6: EntityId;
    requestBlock @7: RequestBlock;
  }
}

struct Aabb3 {
  min @0: Vector3;
  max @1: Vector3;
}

struct BoundPair {
  id @0: EntityId;
  bounds @1: Aabb3;
}

struct TriangleVertexPositions {
  v0 @0: Vector3;
  v1 @1: Vector3;
  v2 @2: Vector3;
}

struct TriangleVertexNormals {
  v0 @0: Vector3;
  v1 @1: Vector3;
  v2 @2: Vector3;
}

struct PixelCoords {
  x @0: Float32;
  y @1: Float32;
}

# TODO: Add simple generics and strongly-type the Vec contents so their lengths are identical.

struct TerrainBlock {
  # Position of each vertex.
  vertexPositions @0: List(Vector3);
  # Vertex normals. These should be normalized!
  vertexNormals @1: List(Vector3);
  # Per-vertex indices into an array in `pixels`.
  pixelCoords @2: List(PixelCoords);
  # Entity IDs for each triangle.
  triangleIds @3: List(EntityId);
  # Per-triangle bounding boxes.
  bounds @4: List(BoundPair);
  # Pixels for this block.
  pixels @5: List(Color3);
}

struct TerrainBlockSend {
  position @0: BlockPosition;
  lod @1: UInt32;
  block @2: TerrainBlock;
}

struct PlayerAdded {
  id @0: EntityId;
  position @1: Vector3;
}

struct UpdatePlayer {
  entity @0: EntityId;
  bounds @1: Aabb3;
}

struct RotatePlayer {
  player @0: EntityId;
  rx @1: Float32;
  ry @2: Float32;
}

struct UpdateMob {
  entity @0: EntityId;
  bounds @1: Aabb3;
}

struct RequestBlock {
  position @0: BlockPosition;
  client @1: ClientId;
  lodIndex @2: UInt32;
}

struct ServerToClient {
  type @0: Int8;
  union {
    leaseId @1: ClientId;
    ping @2: Void;
    playerAdded @3: PlayerAdded;
    updatePlayer @4: UpdatePlayer;
    updateMob @5: UpdateMob;
    updateSun @6: Float32;
    addBlock @7: TerrainBlockSend;
  }
}
