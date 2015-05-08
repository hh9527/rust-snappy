Snappy
------------

[ Forked from https://github.com/thestinger/rust-snappy ]

Snappy bindings for Rust as written in the [The Official Rust Book](https://doc.rust-lang.org/book/ffi.html)

It didn't exist on crates.io, and I needed to use it as a crate!

[Documentation](https://jeffbelgum.github.io/snappy/snappy/)

Usage
-----

Add this to your `Cargo.toml`:

```ini
[dependencies]
snappy = "0.2.0"
```

and this to your crate root:

```rust
extern crate snappy;
```

Installing Snappy
-----------------

The Snappy C++ library can be installed on Mac OS X using homebrew ```brew
install snappy```
