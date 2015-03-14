//! Copy-based serialization functions. We don't use rustc-serialize
//! because it doesn't support bulk copies of Copyable things.

#![feature(collections)]
#![feature(core)]

#[cfg(test)]
extern crate "rustc-serialize" as rustc_serialize;

use std::mem;
use std::num;
use std::raw;

#[derive(Debug)]
pub struct EOF;

pub struct MemStream<'a> {
  data: &'a [u8],
  position: usize,
}

impl<'a> MemStream<'a> {
  pub fn new(data: &'a [u8]) -> MemStream<'a> {
    MemStream {
      data: data,
      position: 0,
    }
  }

  pub fn take(&mut self, len: usize) -> Result<&'a [u8], EOF> {
    if self.data.len() < self.position + len {
      return Err(EOF)
    }

    let old_position = self.position;
    self.position += len;
    let r = &self.data[old_position .. self.position];
    Ok(r)
  }
}

/// Shortcut function for Flatten::emit with `Vec::new()`.
pub fn encode<T>(v: &T) -> Result<Vec<u8>, ()> where T: Flatten {
  let mut r = Vec::new();
  try!(Flatten::emit(v, &mut r));
  Ok(r)
}

/// Shortcut function for Flatten::read.
pub fn decode<T>(data: &[u8]) -> Result<T, EOF> where T: Flatten {
  let mut memstream = MemStream::new(data);
  Flatten::read(&mut memstream)
}

pub fn emit_as_bytes<T>(v: &T, dest: &mut Vec<u8>) -> Result<(), ()> {
  let bytes = unsafe {
    mem::transmute(
      raw::Slice {
        data: v as *const T as *const u8,
        len: mem::size_of::<T>(),
      }
    )
  };

  dest.push_all(bytes);
  Ok(())
}

pub fn of_bytes<'a, T>(bytes: &mut MemStream<'a>) -> Result<T, EOF> where T: Copy {
  let bytes = try!(bytes.take(mem::size_of::<T>()));

  let v = bytes.as_ptr() as *const T;
  let v = unsafe { *v };
  Ok(v)
}

pub trait Flatten {
  fn emit(v: &Self, dest: &mut Vec<u8>) -> Result<(), ()>;
  fn read<'a>(s: &mut MemStream<'a>) -> Result<Self, EOF>;
}

macro_rules! flatten_struct_impl(
  ( $name: ident, $( $member: ident),* ) => {
    impl Flatten for $name {
      fn emit(v: &$name, dest: &mut Vec<u8>) -> Result<(), ()> {
        $( try!(Flatten::emit(&v.$member, dest)); )*
        Ok(())
      }
      fn read<'a>(s: &mut MemStream<'a>) -> Result<$name, EOF> {
        $( let $member = try!(Flatten::read(s)); )*
        Ok($name {
          $( $member: $member, )*
        })
      }
    }
  }
);

impl<T> Flatten for T where T: Copy {
  fn emit(v: &T, dest: &mut Vec<u8>) -> Result<(), ()> {
    emit_as_bytes(v, dest)
  }

  fn read<'a>(v: &mut MemStream<'a>) -> Result<T, EOF> {
    of_bytes(v)
  }
}

impl<T> Flatten for Vec<T> where T: Copy {
  fn emit(v: &Vec<T>, dest: &mut Vec<u8>) -> Result<(), ()> {
    let len: u32;
    match num::cast(v.len()) {
      None => return Err(()),
      Some(l) => len = l,
    }

    try!(Flatten::emit(&len, dest));

    let bytes = unsafe {
      mem::transmute(
        raw::Slice {
          data: v.as_ptr() as *const u8,
          len: v.len() * mem::size_of::<T>(),
        }
      )
    };

    dest.push_all(bytes);
    Ok(())
  }

  fn read<'a>(v: &mut MemStream<'a>) -> Result<Vec<T>, EOF> {
    let len: u32 = try!(Flatten::read(v));
    let len = len as usize;

    let slice = try!(v.take(len * mem::size_of::<T>()));
    let slice: &[T] = unsafe {
      mem::transmute(
        raw::Slice {
          data: slice.as_ptr() as *const T,
          len: len,
        }
      )
    };

    Ok(slice.iter().map(|&x| x).collect())
  }
}

#[cfg(test)]
mod tests {
  extern crate test;

  use super::{Flatten, MemStream, EOF, encode, decode};

  #[derive(Debug, PartialEq, Eq)]
  #[derive(RustcEncodable, RustcDecodable)]
  struct Foo {
    data: Vec<(i32, u64)>,
    t: i8,
  }

  flatten_struct_impl!(Foo, data, t);

  #[derive(Debug, PartialEq, Eq)]
  #[derive(RustcEncodable, RustcDecodable)]
  struct Bar {
    t: u32,
    items: Foo,
  }

  flatten_struct_impl!(Bar, t, items);

  #[derive(Debug, PartialEq, Eq)]
  #[derive(RustcEncodable, RustcDecodable)]
  struct Baz {
    foo: Foo,
    thing: i8,
    bar: Bar,
  }

  flatten_struct_impl!(Baz, foo, thing, bar);

  #[test]
  fn simple_test() {
    let baz =
      Baz {
        foo: Foo {
          data: vec!((1, 1), (2, 4), (3, 255)),
          t: 118,
        },
        thing: 3,
        bar: Bar {
          t: 7,
          items: Foo {
            data: vec!((0, 8), (3, 9), (6, 10)),
            t: -3,
          },
        },
      };
    let encoded = encode(&baz).unwrap();
    println!("encoded baz: {:?}", encoded);
    let rebaz = decode(encoded.as_slice()).unwrap();
    assert_eq!(baz, rebaz);
  }

  #[bench]
  fn bench_encode_decode_binary(_: &mut test::Bencher) {
    let baz =
      Baz {
        foo: Foo {
          data: vec!((1, 1), (2, 4), (3, 255)),
          t: 118,
        },
        thing: 3,
        bar: Bar {
          t: 7,
          items: Foo {
            data: vec!((0, 8), (3, 9), (6, 10)),
            t: -3,
          },
        },
      };
    let encoded = encode(&baz).unwrap();
    let rebaz: Baz = decode(encoded.as_slice()).unwrap();
    test::black_box(rebaz);
  }
}
