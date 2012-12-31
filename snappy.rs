extern mod std;

extern "C" mod snappy {
  fn snappy_compress(input: *const u8,
                     input_length: size_t,
                     compressed: *u8,
                     compressed_length: *size_t) -> c_int;
  fn snappy_uncompress(compressed: *const u8,
                       compressed_length: size_t,
                       uncompressed: *u8,
                       uncompressed_length: *size_t) -> c_int;
  fn snappy_max_compressed_length(source_length: size_t) -> size_t;
  fn snappy_uncompressed_length(compressed: *const u8,
                                compressed_length: size_t,
                                result: *size_t) -> c_int;
  fn snappy_validate_compressed_buffer(compressed: *const u8,
                                       compressed_length: size_t) -> c_int;
}

use libc::c_int;
use libc::size_t;
use ptr::addr_of;
use snappy::*;

pub pure fn validate_compressed_buffer(src: &[u8]) -> bool unsafe {
  snappy_validate_compressed_buffer(vec::raw::to_ptr(src),
                                    src.len() as size_t) == 0
}

pub pure fn compress(src: &[u8]) -> ~[u8] unsafe {
  let srclen = src.len() as size_t;
  let psrc = vec::raw::to_ptr(src);

  let dstlen = snappy_max_compressed_length(srclen);
  let mut dst = vec::from_elem(dstlen as uint, 0u8);
  let pdst = vec::raw::to_ptr(dst);

  let r = snappy_compress(psrc, srclen, pdst, addr_of(&dstlen));
  assert r == 0; // SNAPPY_BUFFER_TOO_SMALL should never occur

  dst.truncate(dstlen as uint);
  dst
}

pub pure fn uncompress(src: &[u8]) -> Option<~[u8]> unsafe {
  let srclen = src.len() as size_t;
  let psrc = vec::raw::to_ptr(src);

  let dstlen: size_t = 0;
  snappy_uncompressed_length(psrc, srclen, addr_of(&dstlen));

  let mut dst = vec::from_elem(dstlen as uint, 0u8);
  let pdst = vec::raw::to_ptr(dst);

  let r = snappy_uncompress(psrc, srclen, pdst, addr_of(&dstlen));

  if r == 0 {
    dst.truncate(dstlen as uint);
    Some(dst)
  } else {
    assert r == 1; // SNAPPY_BUFFER_TOO_SMALL should never occur
    None // SNAPPY_INVALID_INPUT
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn valid() {
    let d: ~[u8] = ~[0xdeu8, 0xadu8, 0xd0u8, 0x0du8];
    let c = compress(d);
    assert validate_compressed_buffer(c);
    let r = option::unwrap(uncompress(c));
    assert r == d;
  }

  #[test]
  fn invalid() {
    let d: ~[u8] = ~[0, 0, 0, 0];
    assert !validate_compressed_buffer(d);
    assert uncompress(d).is_none();
  }
}
