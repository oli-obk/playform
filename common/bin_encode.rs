//! A trait that inherits specialized versions of rustc_serialize's traits, so we can use it as a trait object.
/// The fact that this is necessary makes me so, so sad.

use bincode::rustc_serialize::{EncoderWriter, EncodingError, DecoderReader, DecodingError};
use rustc_serialize;
use std;

/// A trait that inherits specialized versions of rustc_serialize's traits, so we can use it as a trait object.
/// The fact that this is necessary makes me so, so sad.
pub trait BinEncode<'a> : Sized {
  /// See rustc_serialize::Encodable.
  fn encode(&self, s: &mut EncoderWriter<'a, Box<std::io::Write>>) -> Result<(), EncodingError>;
  /// See rustc_serialize::Decodable.
  fn decode(d: &mut DecoderReader<'a, Box<std::io::Read>>) -> Result<Self, DecodingError>;
}

impl<'a, T> BinEncode<'a> for T where
  T: rustc_serialize::Encodable + rustc_serialize::Decodable,
{
  fn encode(&self, s: &mut EncoderWriter<'a, Box<std::io::Write>>) -> Result<(), EncodingError> {
    T::encode(&self, s)
  }

  fn decode(d: &mut DecoderReader<'a, Box<std::io::Read>>) -> Result<Self, DecodingError> {
    T::decode(d)
  }
}
