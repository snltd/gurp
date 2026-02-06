## server

Run Gurp in Server mode.

```
Usage: gurp server [OPTIONS] --config-dir <CONFIG_DIR>

Options:
  -c, --config-dir <CONFIG_DIR>  Where to find host configuration files
  -M, --metrics-to <METRICS_TO>  HTTP POST InfluxDB metrics to this host
  -h, --help                     Print help
```

Gurp uses the same binary for both client and server operation.

The server listens on port 1867, and currently supports only plain HTTP, with
no authentication. The process stays in the foreground.

There is not likely to be a need to speak to the server directly, but the following endpoints are supported:

- `/version` returns the semantic version of the Gurp server as `text/plain`.
- `/status` returns the state of the Gurp server as `text/plain`.
- `/file/<path>` sends back a file, if it exists at `<path>` within the Gurp configuration directory. The MIME type is determined by content of the file.
- `/config/<host>` if the server has a config definition for the given `<host>` it will compile it to a Janet image and send that back as an `application/octet-stream`.


