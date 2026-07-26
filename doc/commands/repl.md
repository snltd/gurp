## repl

Open a Janet REPL with the Gurp library already loaded into the root
environment.

```
Usage: gurp repl

Options:
      --syspath <SYSPATH>
          Set the *syspath* dyn [default: /home/rob/github.com/snltd/gurp]
      --gurp-config-root <GURP_CONFIG_ROOT>
          Set the :gurp-config-root dyn [default: /home/rob/github.com/snltd/gurp]
  -h, --help  Print help
```

`syspath` and `gurp-config-root` both default to your current working directory,
so you will see a different help message.

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
