## v 2.0.0

Items marked [*] are breaking changes.

- Option to compile config into a Janet image, and to apply config from a Janet
  image. This means in client/server mode, the configuration is pulled from the
  server, but evaluated on the client, so you can write and trust code which
  behaves differently on client and server. The old JSON transfer still exists.
- Default to jimage in client/server mode. [*]
- Write a lock file (`/var/run/gurp.lock`) when running in `apply` mode.
- More detailed, more useful error messages.
- Improved client and server telemetry, and tidied metrics namespace. [*]

### Commands

- Remove `show` command. [*]
- Remove `-L` option from `apply` and `compile` commands. [*]
- Add `doers` command, which dumps a list of doers to stdout.
- Add `repl` command, which opens a Janet REPL with the Gurp library loaded into
  the root environment.
- Add `--destroy-everything-you-touch` to `apply` command.
- Add `--as-json` option to apply command, for old compile-on-server behaviour.
- Add `--remove-first` option to act on remove resources first.
- `describe` command gives more information, and its layout adjusts for the
  terminal width.
- `describe` and `doers` will not use ANSI colouring if `gurp` is part of a
  pipeline, or if the user specifies `-C`.
- `apply` has a new `--exec` option which compiles and applies a string of
  Janet config.
- `apply` has a new `--no-lock` option which stops Gurp checking, creating, or
  removing a lock file.

### Doers

- Replace `symlink` doer with `link`, which also handles hard links. [*]
- Changed syntax of `ip-address/ensure`, `ip-properties/ensure`, and
  `ip-interface/ensure` to make them all consistent. [*]
- Helpers like `zone-fs` or `smf-method` are now referred to as `zone/fs`
  and `smf/method`. [*]
- Add `bridge` doer.
- Add `ipfilter` doer.
- Add `ipnat` doer.
- Add `network-flow` doer.
- Add `vlan` doer.
- Add `limitpriv`, `hostid`, `ip-type`, `pool` , `acpi` and `boot-rom` to
  `zone` doer.
- `zone` doer now supports illumos branded zones.
- `zone` doer supports ZST images for bhyve zones (and has been refactored).
- All brands which require an image now use the top-level `:image` key, rather
  than having their own. [*] 
- Add `force-link` to `link` doer, which can turn regular files into links.
- Doer documentation is machine-generated from definition files.
- `file` and `directory` doers now accept numeric IDs for `:owner` and `:group`.
- File backups now work, and the whole `file` doer has been refactored to make
  it easier to work on and expand.
- `file` and `directory` doers now error with a meaningful message if you try
  to ensure a path which already exists, but is of a different type.
- `smf` doer now lets you set service properties.
- Fix bug where removing a service property always produced an error.
- Fix bug where APK packages were never removed.

### Internals

- Huge refactor of the front-end Janet. The old single library file is now fully
  modular, with improved code clarity, better test coverage, and all knots
  untangled. [*]
- Doer parameters, types, and behaviour are now defined in separate definition
  files. These are used to check properties are valid and of the correct types,
  and to generate documentation.
- The front-end checks helper properties and their types.
- Various small bugfixes in front-end.
- Use OpenTelemetry for client and server metrics.
- Remove the internal concept of a "noop". Things either happen or they don't.
- Wherever possible, avoid loading files into memory. This improves client and
  server memory usage.

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
