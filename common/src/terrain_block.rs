//! Data structure for a small block of terrain.

pub const BLOCK_WIDTH: i32 = 8;
pub const TEXTURE_WIDTH: [u32; 4] = [32, 16, 8, 2];
pub const TEXTURE_LEN: [usize; 4] = [
  TEXTURE_WIDTH[0] as usize * TEXTURE_WIDTH[0] as usize,
  TEXTURE_WIDTH[1] as usize * TEXTURE_WIDTH[1] as usize,
  TEXTURE_WIDTH[2] as usize * TEXTURE_WIDTH[2] as usize,
  TEXTURE_WIDTH[3] as usize * TEXTURE_WIDTH[3] as usize,
];

/// Quality across different LODs.
/// Quality is the number of times the noise function is sampled along each axis.
pub const LOD_QUALITY: [u16; 4] = [8, 4, 2, 1];
