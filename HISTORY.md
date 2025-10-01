## v1.1.0 (2025-10-01)

- Add support for Bhyve zones.
- Add `--precompiled` (`-p`) flag to `apply` command, which applies a
  pre-compiled JSON file.
- Fixed bug where `compile` command would fail silently without an output
  format.
- Add `:from-url` and `:with-checksum` to `file` doer, to copy files from a
  remote origin.

## v1.0.0 (2025-09-13)

First release. Covers the basic functionality laid down in the original design.
