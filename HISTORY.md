- Add etherstub support.
- Add new `server` mode which serves compiled (JSON) configurations.
- Add `-s` (`--server`) option to `apply` subcommand to request a compiled
  configuration from a Gurp instance running in `server` mode.
- Smarter handling of conflicting options in `apply` subcommand.
- Don't use colour in logs when not running with a tty.
- Refactor of code which builds lib / host config bundle.
- Minor logging improvements.
- Fix bug which stopped `:from-url` working without `:ignore-pattern` being set.
- Fix bug which blocked downloading large files from server.
- Use canonical paths in Janet lib dyns, for more robust file-finding.
- Better error messages.

## v1.2.0 (2025-10-12)

- Add `vnic` doer, to add and remove VNICs.
- Add `ip-interface` doer, which adds, removes, and sets properties on network
  interfaces.
- Add `ip-address` doer, which allows setting of static and DHCP IP addresses.
  It also manages `addrprop` settings.
- Translate Janet bools into `on` and `off` when defining ZFS properties.

## v1.1.0 (2025-10-01)

- Add support for Bhyve zones.
- Add `--precompiled` (`-p`) flag to `apply` command, which applies a
  pre-compiled JSON file.
- Fix bug where `compile` command would fail silently without an output format.
- Add `:from-url` and `:with-checksum` to `file` doer, to copy files from a
  remote origin.

## v1.0.0 (2025-09-13)

First release. Covers the basic functionality laid down in the original design.
