# control-data

The Gurp DSL provides a `control-data` function. It is used to set predefined
parameters which affect the way a `gurp apply` behaves. Keys are defined as
keywords. All keys are optional and may be defined anywhere in a Gurp config.

Trying to set the same key twice causes a fatal error.

| key              | type      | description                                                                               |
| ---------------- | --------- | ----------------------------------------------------------------------------------------- |
| `:splay-seconds` | [:number] | Tells Gurp to pause by a random number of seconds up to the given maximum before applying |
| `:metrics-to`    | [:string] | Send OpenTelemetry metrics to the given endpoint                                          |
| `:gem-path`      | [:string] | Specify the gem binary used by the gem doer                                               |
| `:logs-to`       | [:string] | Send OpenTelemetry logs to the given endpoint                                             |
| `:strict-hostname` | [:boolean] | Gurp will only apply a config if the top-level `host` name matches the machine's actual hostname |
