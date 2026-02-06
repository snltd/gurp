## repl

Open a Janet REPL with the Gurp library already loaded into the root
environment.

```
Usage: gurp repl

Options:
  -h, --help  Print help
```

The `repl` command puts you "inside" the Gurp front-end, and is useful for
debugging or experimenting with Gurp config.

```
$ gurp repl
repl:1:> (etherstub/ensure "stub0")
{:_id "/NO-ROLE/etherstub/stub0" :name "stub0" :role "NO-ROLE"}
repl:2:> (etherstub/ensure "stub1")
{:_id "/NO-ROLE/etherstub/stub1" :name "stub1" :role "NO-ROLE"}
repl:3:> (pp *collector*)
@{:ensure @{:etherstub @[{:_id "/NO-ROLE/etherstub/stub0" :name "stub0" :role "NO-ROLE"} {:_id "/NO-ROLE/etherstub/stub1" :name "stub1" :role "NO-ROLE"}]} :remove @{}}
nil
```

Janet's embedded REPL does not support nice things like history, tab completion,
or syntax highlighting.
