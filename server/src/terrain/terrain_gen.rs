use capnp::{MessageBuilder, MessageReader, ReaderOptions, MallocMessageBuilder};
use cgmath::{Point, Point2, Point3, Vector, Vector2, Vector3};
use cgmath::Aabb3;
use std::cmp::partial_max;
use std::num;
use std::sync::Mutex;

use common::block_position::BlockPosition;
use common::color::Color3;
use common::communicate::{aabb3, bound_pair, color3, entity_id, pixel_coords, vector3, terrain_block};
use common::entity::EntityId;
use common::id_allocator::IdAllocator;
use common::lod::LODIndex;
use common::stopwatch::TimerSet;
use common::terrain_block::{BLOCK_WIDTH, LOD_QUALITY, TEXTURE_WIDTH};

use opencl_context::CL;
use terrain::heightmap::HeightMap;
use terrain::texture_generator::TerrainTextureGenerator;
use terrain::tree_placer::TreePlacer;

pub fn generate_block(
  timers: &TimerSet,
  cl: &CL,
  id_allocator: &Mutex<IdAllocator<EntityId>>,
  heightmap: &HeightMap,
  texture_generator: &TerrainTextureGenerator,
  treemap: &TreePlacer,
  position: &BlockPosition,
  lod_index: LODIndex,
) -> MallocMessageBuilder {
  timers.time("update.generate_block", || {
    let mut vertex_positions: Vec<Point3<f32>> = Vec::new();
    let mut vertex_normals: Vec<Vector3<f32>> = Vec::new();
    let mut pixel_coords: Vec<Point2<f32>> = Vec::new();
    let mut triangle_ids: Vec<EntityId> = Vec::new();
    let mut bounds: Vec<(EntityId, Aabb3<f32>)> = Vec::new();
    let mut pixels: Vec<Color3<f32>> = Vec::new();

    let position = position.to_world_position();

    let lateral_samples = LOD_QUALITY[lod_index.0 as usize] as u8;
    let sample_width = BLOCK_WIDTH as f32 / lateral_samples as f32;

    let mut any_tiles = false;
    for dx in range(0, lateral_samples) {
      let dx = dx as f32;
      for dz in range(0, lateral_samples) {
        let dz = dz as f32;
        let tex_sample =
          TEXTURE_WIDTH[lod_index.0 as usize] as f32 / lateral_samples as f32;
        let tex_coord = Point2::new(dx, dz).mul_s(tex_sample);
        let tile_position = position.add_v(&Vector3::new(dx, 0.0, dz).mul_s(sample_width));
        if add_tile(
          timers,
          heightmap,
          treemap,
          id_allocator,
          &mut vertex_positions,
          &mut vertex_normals,
          &mut pixel_coords,
          &mut triangle_ids,
          &mut bounds,
          sample_width,
          tex_sample,
          &tile_position,
          tex_coord,
          lod_index,
        ) {
          any_tiles = true;
        }
      }
    }

    if any_tiles {
      pixels =
        texture_generator.generate(
          cl,
          position.x as f32,
          position.z as f32,
        );
    }

    let vertex_positions_len = num::cast(vertex_positions.len()).unwrap();
    let vertex_normals_len = num::cast(vertex_normals.len()).unwrap();
    let pixel_coords_len = num::cast(pixel_coords.len()).unwrap();
    let triangle_ids_len = num::cast(triangle_ids.len()).unwrap();
    let bounds_len = num::cast(bounds.len()).unwrap();
    let pixels_len = num::cast(pixels.len()).unwrap();

    let mut vertex_positions = vertex_positions.into_iter();
    let mut vertex_normals = vertex_normals.into_iter();
    let mut pixel_coords = pixel_coords.into_iter();
    let mut triangle_ids = triangle_ids.into_iter();
    let mut bounds = bounds.into_iter();
    let mut pixels = pixels.into_iter();

    capnpc_new!(
      terrain_block::Builder =>
      [from_fn init_vertex_positions vertex_positions_len => |mut dest: vector3::Builder| {
        let p: Point3<f32> = vertex_positions.next().unwrap();
        capnpc_init!(dest => [set_x p.x] [set_y p.y] [set_z p.z]);
      }]
      [from_fn init_vertex_normals vertex_normals_len => |mut dest: vector3::Builder| {
        let p: Vector3<f32> = vertex_normals.next().unwrap();
        capnpc_init!(dest => [set_x p.x] [set_y p.y] [set_z p.z]);
      }]
      [from_fn init_pixel_coords pixel_coords_len => |mut dest: pixel_coords::Builder| {
        let p: Point2<f32> = pixel_coords.next().unwrap();
        capnpc_init!(dest => [set_x p.x] [set_y p.y]);
      }]
      [from_fn init_triangle_ids triangle_ids_len => |mut dest: entity_id::Builder| {
        let id: EntityId = triangle_ids.next().unwrap();
        capnpc_init!(dest => [set_id id.0]);
      }]
      [from_fn init_bounds bounds_len => |mut dest: bound_pair::Builder| {
        let (id, b): (EntityId, Aabb3<f32>) = bounds.next().unwrap();
        capnpc_init!(dest =>
          [init_id => [set_id id.0]]
          [init_bounds =>
            [init_min => [set_x b.min.x] [set_y b.min.y] [set_z b.min.z]]
            [init_max => [set_x b.max.x] [set_y b.max.y] [set_z b.max.z]]
          ]
        );
      }]
      [from_fn init_pixels pixels_len => |mut dest: color3::Builder| {
        let c: Color3<f32> = pixels.next().unwrap();
        capnpc_init!(dest => [set_r c.r] [set_g c.g] [set_b c.b]);
      }]
    )
  })
}

fn add_tile<'a>(
  timers: &TimerSet,
  hm: &HeightMap,
  treemap: &TreePlacer,
  id_allocator: &Mutex<IdAllocator<EntityId>>,
  vertex_positions: &mut Vec<Point3<f32>>,
  vertex_normals: &mut Vec<Vector3<f32>>,
  pixel_coords: &mut Vec<Point2<f32>>,
  triangle_ids: &mut Vec<EntityId>,
  bounds: &mut Vec<(EntityId, Aabb3<f32>)>,
  sample_width: f32,
  tex_sample: f32,
  position: &Point3<f32>,
  tex_coord: Point2<f32>,
  lod_index: LODIndex,
) -> bool {
  let half_width = sample_width / 2.0;
  let center = hm.point_at(position.x + half_width, position.z + half_width);

  if position.y >= center.y || center.y > position.y + BLOCK_WIDTH as f32 {
    return false;
  }

  timers.time("update.generate_block.add_tile", || {
    let normal_delta = sample_width / 2.0;
    let center_normal =
      hm.normal_at(normal_delta, position.x + half_width, position.z + half_width);

    let x2 = position.x + sample_width;
    let z2 = position.z + sample_width;

    let ps: [Point3<_>; 4] = [
      hm.point_at(position.x, position.z),
      hm.point_at(position.x, z2),
      hm.point_at(x2, z2),
      hm.point_at(x2, position.z),
    ];

    let ns: [Vector3<_>; 4] = [
      hm.normal_at(normal_delta, position.x, position.z),
      hm.normal_at(normal_delta, position.x, z2),
      hm.normal_at(normal_delta, x2, z2),
      hm.normal_at(normal_delta, x2, position.z),
    ];

    let ts: [Point2<_>; 4] = [
      tex_coord.add_v(&Vector2::new(0.0, 0.0)),
      tex_coord.add_v(&Vector2::new(0.0, tex_sample)),
      tex_coord.add_v(&Vector2::new(tex_sample, tex_sample)),
      tex_coord.add_v(&Vector2::new(tex_sample, 0.0)),
    ];

    let center_tex_coord =
      tex_coord.add_v(&Vector2::new(tex_sample, tex_sample).mul_s(1.0 / 2.0));

    macro_rules! place_terrain(
      ($v1: expr,
        $v2: expr,
        $minx: expr,
        $minz: expr,
        $maxx: expr,
        $maxz: expr
      ) => ({
        let v1 = &ps[$v1];
        let v2 = &ps[$v2];
        let n1 = &ns[$v1];
        let n2 = &ns[$v2];
        let t1 = &ts[$v1];
        let t2 = &ts[$v2];
        let maxy = partial_max(v1.y, v2.y);
        let maxy = maxy.and_then(|m| partial_max(m, center.y));
        let maxy = maxy.unwrap();

        let id = id_allocator.lock().unwrap().allocate();

        vertex_positions.push_all(&[*v1, *v2, center]);
        vertex_normals.push_all(&[*n1, *n2, center_normal]);
        pixel_coords.push_all(&[*t1, *t2, center_tex_coord]);
        triangle_ids.push(id);
        bounds.push((
          id,
          Aabb3::new(
            Point3::new($minx, v1.y, $minz),
            Point3::new($maxx, maxy, $maxz),
          ),
        ));
      });
    );

    let polys =
      (LOD_QUALITY[lod_index.0 as usize] * LOD_QUALITY[lod_index.0 as usize] * 4) as usize;

    let centr = center; // makes alignment nice
    place_terrain!(0, 1, ps[0].x, ps[0].z, centr.x, ps[1].z);
    place_terrain!(1, 2, ps[1].x, centr.z, ps[2].x, ps[2].z);
    place_terrain!(2, 3, centr.x, centr.z, ps[2].x, ps[2].z);
    place_terrain!(3, 0, ps[0].x, ps[0].z, ps[3].x, centr.z);

    if treemap.should_place_tree(&centr) {
      treemap.place_tree(
        centr,
        id_allocator,
        vertex_positions,
        vertex_normals,
        pixel_coords,
        triangle_ids,
        bounds,
        lod_index,
      );
    }

    true
  })
}
