[![Test Rust](https://github.com/snltd/gurp/actions/workflows/test-rust.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-rust.yml) [![Janet Tests](https://github.com/snltd/gurp/actions/workflows/test-janet.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-janet.yml)
# gurp

Gurp is an almost certainly doomed attempt to write an illumos configuration
management tool with a Lisp front-end.

## Design

A user writes machine configurations using a thin
[Janet](https://janet-lang.org/) DSL. Resources such as Unix users, SMF
services, or ZFS datasets have their properties described as Janet tables, but
can, of course, be
wrapped in, or contain, arbitrary Janet code. Resources may reference properties
of other resources.

### Important

- Configuration is Janet. It seems such a natural fit for this domain.
- Prefer speed to flexibility.
- Coverage of everything I need in OmniOS.
- Clear reporting at the end of a run.

### Maybe

- ~~You (or some program really) compiles a single binary containing Janet, gurp's
  "doers", and the machine configuration. You can then plonk that binary on your
  host and run it. Or perhaps a `.jimage` file, which would only require the
  `janet` binary.~~ This isn't happening now. The back-end will be Rust. 

### Not Important

- Any kind of back-end or server.
- Clever ordering of resources - I might take a crude multiple-run approach.
- Coverage of anything I don't need in OmniOS.

### Not Happening

- Coverage of anything that isn't OmniOS.

## NOTES

Currently needs `CFLAGS="-D__EXTENSIONS__ -std=c99" cargo build` to compile.
