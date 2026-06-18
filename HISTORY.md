- Add `(metadata)` function and supporting front-end code.

## v2.1.0 (2026-06-16)

- Add `--logs-to` to `apply` and `server` commands to optionally send
  OpenTelemetry logs,
- Add `completions` command to generate shell completions.
- Add `zone-brand` fact.
- Add `num-field-sort` function to DSL.
- Write JSON summary to `/var/log` at end of run.

## v2.0.1 (2026-05-21)

- Fix bug in publisher doer which errored when removing origins or mirrors.

## v2.0.0 (2026-05-20)

Items marked [*] are breaking changes.

- Option to compile config into a Janet image, and to apply config from a Janet
  image. This means in client/server mode, the configuration is pulled from the
  server, but evaluated on the client, so you can write and trust code which
  behaves differently on client and server. The old JSON transfer still exists.
- Default to jimage in client/server mode. [*]
- Write a lock file (`/var/run/gurp.lock`) when running in `apply` mode.
- More detailed, more useful error messages.
- Improved client and server telemetry, and tidied metrics namespace. [*]
- New `/api/v1/gurp-binary` path serves up the currently-running Gurp binary,
  for easier client bootstrapping.

### Commands

- Remove `show` command. [*]
- Remove `-L` option from `apply` and `compile` commands. [*]
- Add `doers` command, which dumps a list of doers to stdout.
- Add `repl` command, which opens a Janet REPL with the Gurp library loaded into
  the root environment.
- Add `--destroy-everything-you-touch` to `apply` command.
- Add `--as-json` option to `apply` command, for old compile-on-server
  behaviour.
- Add `--remove-first` option to `apply`, to act on remove resources first.
- Add `--only <REGEX>` option to `apply`, to apply only resources whose IDs
  match the given Rust regex.
- Add `--exec` to `apply`, which compiles and applies a string of Janet config.
- Add `--no-lock` to `apply`, which stops Gurp checking, creating, or removing a
  lock file.
- `describe` command gives more information, and its layout adjusts for the
  terminal width.
- `describe` and `doers` will not use ANSI colouring if `gurp` is part of a
  pipeline, or if the user specifies `-C`.

### Doers

- Replace `symlink` doer with `link`, which also handles hard links. [*]
- Changed syntax of `ip-address/ensure`, `ip-properties/ensure`, and
  `ip-interface/ensure` to make them all consistent. [*]
- Helpers like `zone-fs` or `smf-method` are now referred to as `zone/fs` and
  `smf/method`. [*]
- Add `bridge` doer.
- Add `ipfilter` doer.
- Add `ipnat` doer.
- Add `network-flow` doer.
- Add `vlan` doer.
- Add `system-cert` doer.
- Add `limitpriv`, `hostid`, `ip-type`, `pool` , `acpi` and `boot-rom` to `zone`
  doer.
- `zone` doer now supports illumos branded zones.
- `zone` doer supports ZST images for bhyve zones (and has been refactored).
- Resolvers and domain now option in `lx` and native `zone` configs.
- All brands which require an image now use the top-level `:image` key, rather
  than having their own. [*]
- Add `force-link` to `link` doer, which can turn regular files into links.
- Doer documentation is machine-generated from definition files.
- When running in client/server mode, the `file` doer is now able to ask the
  server for the Blake3 hash of a file, only retrieving it if needs to. This
  reduces memory usage on the client, and network traffic.
- `file` and `directory` doers now accept numeric IDs for `:owner` and `:group`.
- File backups now work, and the whole `file` doer has been refactored to make
  it easier to work on and expand.
- `file` and `directory` doers now error with a meaningful message if you try to
  ensure a path which already exists, but is of a different type.
- `smf` doer now lets you set service properties.
- Fix bug where removing a service property always produced an error.
- Fix bug where APK packages were never removed.

### DSL

- Remove the `(hostname)` function. Use `(fact :hostname)` instead. [*]
- Change `run-cmd` so it only allows execution of certain allowed commands [*]
- Add `run-safe-cmd` which allows execution of exactly specified commands.
- Add `run-any-cmd` which allows execution of any command in the compilation
  phase unless disallowed by sandboxing rules.
- Add `facts` function, which gives
  [a small amount of system information](/doc/facts.md).
- Add `tabular-output->struct` function, which converts tabular command output
  such as that produced by `zoneadm list` or `ipadm` into Janet structs.
- References can be used in helpers like `smf/method`. (But you cannot reference
  properties of helpers.)

### Internals

- Huge refactor of the front-end Janet. The old single library file is now fully
  modular, with improved code clarity, better test coverage, and all knots
  untangled. [*]
- `indoc` is no longer a macro which binds a string to a name. It is now a
  function which returns a string. [*]
- Run the embedded Janet interpreter with sandboxing. This severely limits the
  actions the Janet code can take when it is evaluated. [*]
- Doer parameters, types, and behaviour are now defined in separate definition
  files. These are used to check properties are valid and of the correct types,
  and to generate documentation.
- The front-end checks helper properties and their types.
- Various small bugfixes in front-end.
- Use OpenTelemetry for client and server metrics.
- Remove the internal concept of a "noop". Things either happen or they don't.
- Wherever possible, avoid loading files into memory. This improves client and
  server memory usage.
- Added context to errors throughout the codebase. As most errors end up being
  seen by the user, this is very helpful.

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
