//! Defines the messages passed between client and server.

use std::default::Default;
use std::ops::Add;

pub use communicate_capnp::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(RustcDecodable, RustcEncodable)]
/// Unique ID for a loaded entity.
pub struct ClientId(pub u32);

impl Default for ClientId {
  fn default() -> ClientId {
    ClientId(0)
  }
}

impl Add<u32> for ClientId {
  type Output = ClientId;

  fn add(self, rhs: u32) -> ClientId {
    let ClientId(i) = self;
    ClientId(i + rhs)
  }
}
