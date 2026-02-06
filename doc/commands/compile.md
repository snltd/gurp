## compile

Compile a Janet host description.

```
Usage: gurp compile [OPTIONS] --format <FORMAT> <HOST_CONFIG_FILE>

Arguments:
  <HOST_CONFIG_FILE>  Host configuration file

Options:
  -N, --line-no                    When displaying compiled config, number lines
  -d, --dump-config                Dump intermediate config files to stdout
  -f, --format <FORMAT>            Output in the given format: 'jimage' or 'json' [default: json]
  -o, --output-file <OUTPUT_FILE>  Output file for compiled config (required for jimage, optional for others)
  -h, --help                       Print help
```

Takes a Janet host configuration, specified as the only argument, and produces
a representation of it in one of three formats.

- `janet` produces a single, finalised, Janet struct. You can prettify this with the `-N`
and `-c` options. It may be useful for debugging.
- `json` produces a single long JSON string.
This structure can be applied on another machine, but be aware that is static and finalised. If your config inspected the local host in anyway, applying to a different host may produce something unexpected.
Gurp does not prettify the JSON: if you want to do that, pipe it through `jq`.
- `jimage` builds [a Janet image](https://janet.guide/compilation-and-imagination/). This is an intermediate representation of desired state: a recipe which, when applied, will generate a finalised JSON structure. When this is applied on another host, any host-specific config will be calculated correctly. When you use Gurp in client-server mode, jimages are sent across the wire. Images are a binary format, so Gurp will not write them to standard out: you must also specify and output file with `-o`.


