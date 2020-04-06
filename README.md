# ddelta-rs
[![Crates.io](https://img.shields.io/crates/v/ddelta.svg)](https://crates.io/crates/ddelta)
[![Docs.rs](https://docs.rs/ddelta/badge.svg)](https://docs.rs/ddelta)

A rust port of [ddelta], which is a streaming and more efficient version
of [bsdiff]. The output created by this program is sometimes (when using
[`generate`]) compatible with the original C tool, [ddelta], but not
with [bsdiff]. This library may use up to 5 times the old file size +
the new file size, (5 × min(o, 2^31-1) + min(n, 2^31-1)), up to 12GiB.
To control this, see the `chunk_sizes` parameter of
[`generate_chunked`]. **Note**: the patches created by program should be
compressed. If not compressed, the output may actually be larger than
just including the new file. You might want to feed the patch file
directly to an [encoder][XzEncoder], and read via a [decoder
implementing a compression algorithm][XzDecoder] to not require much
disk space. Additionally, no checksum is performed, so you should
strongly consider doing a checksum of at least either the old or new
file once written.

## Features

This crate optionally supports compiling the c library, divsufsort,
which is enabled by default. A Rust port is available; however, it has
worse performance than the C version. If you'd like to use the Rust
version instead, for example if you don't have a C compiler installed,
add `default-features = false` to your Cargo.toml, i.e.

```toml
[dependencies]
ddelta = { version = "0.1.0", default-features = false }
```

[ddelta]: https://github.com/julian-klode/ddelta
[bsdiff]: http://www.daemonology.net/bsdiff/
[XzEncoder]: https://docs.rs/xz2/*/xz2/write/struct.XzEncoder.html
[XzDecoder]: https://docs.rs/xz2/*/xz2/read/struct.XzDecoder.html

[`generate`]: https://docs.rs/ddelta/*/ddelta/fn.generate.html
[`generate_chunked`]: https://docs.rs/ddelta/*/ddelta/fn.generate_chunked.html
