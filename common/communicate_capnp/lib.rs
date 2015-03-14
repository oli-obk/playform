extern crate capnp;

pub mod src_capnp {
  include!(concat!(env!("OUT_DIR"), "/src_capnp.rs"));
}

pub use src_capnp::*;
