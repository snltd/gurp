- Removed `show` command. (Breaking change.)
- Add `repl` command, which opens a Janet REPL with the Gurp library loaded into the root environment.
- Removed the "show" command.
- Option to compile config into a Janet image, and to apply config from a Janet
  image. This means in client/server mode, the configuration is pulled from the
  server, but evaluated on the client, so you can write and trust code which
  behaves differently on client and server. The old JSON transfer still exists.
- Default to jimage in client/server mode. (Breaking change.)
- Add `--as-json` option to client mode to get old behaviour.
- Add `network-flow` doer, a wrapper around `flowadm(8)`.
- Add `vlan` doer.
- Add `ipnat` doer.
- Add `resources` command, which dumps a list of doers to stdout.
- Add `--destroy-everything-you-touch` to `apply` command.
- Add `limitpriv`, `hostid`, `ip-type`, `pool` to `zone` doer.

## v 1.4.0 (2025-11-22)

- Add `route` doer, which manages persistent routes.
- Add `ip-properties` doer to manage top-level IP properties.
- Add `(repeated-line-file)` helper function.
- Add `(smf-dependency)` and `(smf-dependent)` to the `smf` doer, allowing the
  user to define dependencies beyond the hardcoded ones.
- Show content of new files when using `--dump-diff`. Useful for testing
  dynamically generated content.
- When fetching config from a server, retry with an exponential backoff.

## v 1.3.0 (2025-11-15)

- Add `etherstub` doer.
- Add `server` mode which serves compiled (JSON) configurations.
- Add `-s` (`--server`) option to `apply` subcommand to request a compiled
  configuration from a Gurp instance running in `server` mode.
- When running as a server, push OpenTelemetry metrics.
- Smarter handling of conflicting options in `apply` subcommand.
- Improve `zone` doer's `:copy-in`. The target can now be a directory (add a
  trailing `/`; target directories are created as required.
- Don't use colour in logs when not running with a tty.
- Refactor of code which builds lib / host config bundle.
- Fix bug which stopped `file` doer's `:from-url` working without
  `:ignore-pattern` being set.
- Fix bug which blocked downloading large files from server.
- Use canonical paths in Janet lib dyns, for more robust file-finding.
- Minor logging improvements.

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
