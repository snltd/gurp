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
  [Janet library](../janet_src/lib/gurp.janet) which provides front-end
  functionality. With this option you can provide a path to your own library,
  which is useful for developing new features.

- `-d, --dump-config` Gurp turns its input into various other formats, such as
  JSON, XML, and `zonecfg` config. With this flag, it will dump all those
  configs to standard out. This is independent of the log level, but can be very
  useful when debugging.

- `-C, --colour` When dumping configs, use syntax colouring where possible. This
  only works for Janet.

- `-N, --line-no` When dumping configs, number the lines.

- `-M, --metrics-to <METRICS_TO>` HTTP POST InfluxDB-format metrics to this
  host. At the moment ust sends the run duration, the number of resources, and
  the number of changes.

## `gurp compile`

```sh
$ gurp compile [options] <host-config-file>
```

This compiles your Janet configuration. With no options successful compilation
produces no output, so you can use it as a linter. The following options are
supported.

- `-f, --format <FORMAT>` Use this option if you want to see the compiled
  output. If `FORMAT` is `janet` you will get a Janet struct which defines the
  entire configuration. If you specify `json`, you will get that struct
  converted to JSON.

- `-N, --line-no` Used in conjunction with `-f`, numbers the lines.

- `-L, --gurp-lib <GURP_LIB_PATH>` Specify a gurp Janet library, just like
  `apply`.

## `gurp describe`

```sh
$ gurp describe <RESOURCE>
```

Describes the spec for the given resource type. For instance:

```
$ gurp describe file-line
(file-line/ensure)
No mandatory keys
optional keys
  with        string          Counterpart to :replace
  apply-to    string          Which matches to act on when replacing: 'first', 'last', 'all'
  replace     string          Pattern to replace. Rust regex
  insert-at   number          If a new line must be added, it will go at this index
  line        string          The line which must exist

(file-line/remove)
mandatory keys
  pattern     string          The line or pattern which must be removed
optional keys
  match       string          How to match the line: 'exact', 'starts_with', 'ends_with',
                              'contains', or 'regex'. Default 'exact'
  apply-to    string          Which matches to act on: 'first', 'last', 'all'. Default 'all'
```

## `gurp resources`

Dumps a list of all the doers. For further information on any of them, use
`gurp show`.

## `gurp show`

Dumps the built-in library and default settings to standard out.

```
$ gurp show library | sed 5q
# We only need this import for testing. Gurp bundles the defaults file.
(if-not (get (curenv) (symbol :default-protos))
  (use ./defaults))

(defn new-collector
```

```
$ gurp show defaults | sed 5q
(def default-protos
  {:ensure
   {:cron
    {:hour "*"
     :minute "*"
```

## `gurp server`

Runs Gurp in server mode. It listens on port 1867, and you connect to it with
`gurp apply --server`.

Note that configuration is compiled ON THE SERVER, so if your Janet contains any
conditional logic which refers to system state, that's the host it will be
looking at.
