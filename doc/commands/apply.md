# Gurp Command-Line Usage

Gurp has a number of commands.

## `gurp apply`

`apply` aligns the state of a host with that defined in a configuration file.

You can apply locally stored Janet configurations:

```sh
$ gurp apply [options] <host-config-file>
```

a pre-compiled Janet or JSON configuration:

```sh
$ gurp apply [options] --precompiled <host-config-file>
```

or fetch configuration from a remote server:

```sh
$ gurp apply [options] --server <gurp-server>
```

Gurp uses the [Tracing framework](https://crates.io/crates/tracing) for logging.
This means you can control the log level through the `RUST_LOG` environment
variable. It defaults to `info`, but for debug logs, run

```sh
$ RUST_LOG=debug gurp apply file.janet
```

When bootstrapping zones, the value of `RUST_LOG` is passed through to the zone
being built.

There are options:

- `-n, --noop` Say what would happen, without actually doing it. Because Gurp
  does not include a doer for arbitrary commands, it can make a pretty good
  guess at required changes. All logging is the same as if you were doing a real
  run.

- `-L, --gurp-lib <GURP_LIB_PATH>` Gurp has a built-in
  [Janet library](../janet/lib/gurp.janet) which provides front-end
  functionality. With this option you can provide a path to your own library,
  which is useful for developing new features.

- `-d, --dump-config` Gurp turns its input into various other formats, such as
  JSON, XML, and `zonecfg` config. With this flag, it will dump all those
  configs to standard out. This is independent of the log level, but can be very
  useful when debugging.

- `-D --dump-diffs` When the content of a text file changes, output a diff.

- `-C, --colour` When dumping configs, use syntax colouring where possible. This
  only works for Janet.

- `-N, --line-no` When dumping configs, number the lines.

- `-M, --metrics-to <METRICS_TO>` HTTP POST InfluxDB-format metrics to this
  host. At the moment ust sends the run duration, the number of resources, and
  the number of changes.

- `-s, --server <SERVER>` Fetches pre-compiled JSON config, over HTTP, from a
  Gurp server.

- `-H, --hostname <HOSTNAME>` Used in conjunction with `--server`, lets you
  request config for a specific host.

- `-p, --precompiled` Applies a precompiled JSON config, rather than compiling
  from Janet source.

- `--destroy-everything-you-touch` Turns all `ensure` resources into `remove`s.
  Useful for cleaning up during development, but should be used with extreme
  caution.
