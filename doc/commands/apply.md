## apply

```
Configure this host according to the supplied configuration

Usage: gurp apply [OPTIONS] [HOST_CONFIG_FILE]

Arguments:
  [HOST_CONFIG_FILE]  Host configuration file

Options:
  -M, --metrics-to <METRICS_TO>       HTTP POST OpenTelemetry metrics to this host
  -s, --server <SERVER>               Get config from a Gurp server
  -H, --hostname <HOSTNAME>           Hostname to use when fetching config from server
  -L, --logs-to <LOGS_TO>             HTTP POST OpenTelemetry logs to this host
  -p, --precompiled                   Use a pre-compiled JSON config, which may be local or remote
  -i, --image                         Use a local pre-compiled Janet jimage as config
  -n, --noop                          Say what would happen, without actually doing it
  -D, --define <DEFINE>               Define a constant which can be accessed from config
      --dump-configs                  Dump intermediate config files to stdout
      --dump-diffs                    When files change, dump diffs to stdout
  -C, --colour                        When dumping configs or diffs, use syntax colouring where supported
  -N, --line-no                       When dumping configs, number lines
      --destroy-everything-you-touch  Turn all ensures into removes. Use with extreme caution
  -e, --exec <EXEC>                   Execute a literal snippet of Janet config
      --no-lock                       Do not check for or use a lockfile
      --remove-first                  Run remove actions BEFORE ensure actions
      --only <ONLY>                   Only apply resources whose IDs match this regex
  -h, --help                          Print help
```

`apply` aligns the state of a host with that defined host configuration. That
configuration may be any of:

- one or more Janet files written in the Gurp DSL.
- a single Janet file output compiled from the above, by
  `gurp compile --format=janet`.
- a single JSON structure, normally created with `gurp compile --format=json`,
  though you could theoretically create it however you liked.
- a binary jimage file created manually with `gurp compile --format=jimage`, or
  by a Gurp server as a result of a `/v1/client` API request.

There are options:

- `-s, --server <SERVER>` Fetches configuration, over HTTP, from a Gurp server.
  Normally this is sent in a single jimage binary format which is executed on
  the client, producing the final configuration which is then applied.
- `-J, --as-json` If this is provided with `--server`, the server will compile
  the client's configuration itself, and send the finalised JSON, which the
  client applies.
- `-H, --hostname <HOSTNAME>` ordinarily an apply with `--server` will fetch
  configuration for a host matching the client's own hostname. This option lets
  you request config for a different host. It can be useful for applying the
  same configuration across hosts.
- `-p, --precompiled` Applies a precompiled JSON config, rather than compiling
  from Janet source.
- `-i, --image` Compiles and applies a jimage config, rather than compiling from
  Janet source.
- `-n, --noop` Say what would happen, without actually doing it. Because Gurp
  does not include a doer for arbitrary commands, it can make a meaningful dry
  run. All logging is the same as if you were doing a real apply.
- `--dump-configs` Gurp turns its input into various other formats, such as
  JSON, XML, YAML, INI files, `zonecfg` config and so-on. With this flag, it
  will dump all those configs to standard out. This is independent of the log
  level, but can be very useful when debugging.
- `--dump-diffs` When the content of a text file changes, output a diff.
- `-C, --colour` When dumping configs, use syntax colouring where possible.
- `-N, --line-no` When dumping configs, number the lines.
- `-M, --metrics-to <HOST>` HTTP POST OpenTelemetry metrics to `HOST`.
- `-L, --logs-to <HOST>` HTTP POST OpenTelemetry logs to `HOST` with `gurp` as
  the service name.
- `--destroy-everything-you-touch` Turns all `ensure` resources into `remove`s.
  Useful for cleaning up during development, but should be used with extreme
  caution. It has no short value, so you can't type it by accident.
- `--no-lock` makes Gurp not check for, create, or remove its runtime lock file.
- `--exec` lets you apply a literal string of Janet config to the local host.
  (Assuming you have permission to do so!)
- `--remove-first` Normally Gurp runs all `ensure` resources, then all `remove`
  resources. With this flag, that order is reversed.
- `-O, --only <REGEX>` Makes Gurp only apply resources whose IDs match the given
  Rust regex.
- `-D, --define <DEFINE>` Can be used multiple time, with the values used to
  build a Janet struct, `gurp-user-defs` with global scope, visible during the
  compile phase. If `DEFINE` is of the form `key=value`, the struct gets `key`
  as a symbol and `value` as a string. If `DEFINE` is `key` only, the struct
  gets `key` as a symbol, with a value of boolean `true`.

Gurp uses the [Tracing framework](https://crates.io/crates/tracing) for logging.
This means you can control the log level through the `RUST_LOG` environment
variable. It defaults to `info`, but for debug logs, run

```sh
$ RUST_LOG=debug gurp apply file.janet
```

When bootstrapping zones, the value of `RUST_LOG` is passed through to the zone
being built.
