## Metrics

Gurp uses the OpenTelemetry framework to send metrics in client and server mode.

At the time of writing the backend port must be `8428`, and the API path is
hardcoded to `/opentelemetry/v1/metrics`. Transfer is over HTTP.

### Client Metrics

| metric path              | unit         | metric type | extra labels            | description                               |
| ------------------------ | ------------ | ----------- | ----------------------- | ----------------------------------------- |
| `gurp.apply.duration_ms` | milliseconds | histogram   | `apply`: `ok` or `fail` | Time taken for the entire apply operation |
| `gurp.apply.resources`   |              | gauge       |                         | Number of Gurp-managed resources          |
| `gurp.apply.changes`     |              | gauge       |                         | Number of resources changed               |
| `gurp.apply.rss_bytes`   | bytes        | gauge       |                         | Memory used by Gurp at the end of the run |

### Server Metrics

| metric path                       | unit  | metric type                  | extra labels | description                                             |
| --------------------------------- | ----- | ---------------------------- | ------------ | ------------------------------------------------------- |
| `gurp.server.rss_bytes`           | bytes | gauge                        |              | RSS memory used by server process, according to `/proc` |
| `gurp.server.requests_total`      |       | counter                      |              | Number of requests served                               |
| `gurp.server.request_duration_ms` |       | Time taken to serve requests |              |                                                         |
