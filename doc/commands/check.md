## check

Check the validity of a Janet config.

```
Usage: gurp check [OPTIONS] <PATH>

Arguments:
  <PATH>

Options:
  -M, --metrics-to <METRICS_TO>  HTTP POST OpenTelemetry metrics to this host
  -L, --logs-to <LOGS_TO>        HTTP POST OpenTelemetry logs to this host
  -h, --help                     Print help
```

Flychecks the given file. If the file is syntactically valid, exits `0`; if
there is a problem, the Janet error is dumped to stdout and Gurp exits `1`.

Options are ignored.
