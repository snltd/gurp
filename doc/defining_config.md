## How to Configure a machine

> This is a work-in-progress and subject to (possibly complete) change.

Config is written in [Janet](https://janet-lang.org) which is a Lisp (or not,
depending on how much of a Lisp nerd you are). This means you can write
configuration as code, rather than configuration as configuration file. You can
do anything your Janet chops allow, provided you end up calling the resource
definition functions. A couple of things to bear in mind.

- Janet does not hoist functions. You can't refer to something until you've
  defined it.

## Host Definition

A host definition starts with a `host` definition. This definition references
three modules.

```clojure
(host "example-host"
  (basenode)
  (user-tools)
  (security))
```

The host name isn't used for anything other than logging. It won't change the
actual hostname.

Note that we didn't need to `include` or `use` any library file to get access to
the `host`. Gurp has a hardcoded library file, in which the `host` macro is
defined. It is automatically injected at the top of your code. If you run `gurp`
with `--debug` you will see the full augmented file which is compiled.

## Roles

A host needs one or more roles. If you want, you can put the roles in the same
file as the host definition, but you'll need to put them above the `(host)`
because of the warning I gave you earlier.

Typically you'll put roles in their own files, as you would with any other
config management tool. They follow the normal Janet importing rules, so the
`basenode` role could be:

- `basenode.janet`
- `basenode/init.janet` (which could `use` other files in that directory)
- `basenode.jimage` (if you don't know what this is, you don't need to)

The include path is automagically manipulated so roles will be found at the same
level as your host definition file. If you want to reference them elsewhere,
you'll need to `(use)` them.

```clojure
(role "my-role"
      (pkg/ensure "/ooce/developer/rust")
      (pkg/remove "/ooce/developer/go")
      (file/ensure "/etc/application/config.txt"
                   :label "app-config"
                   :owner "root"
                   :content (string "config values for " (this-host))
      (directory/ensure "/etc/application"
                        :owner :/my-role/file/app-config/owner
                        :group "engineering"
                        :mode "0750"))
```

## Resources

Resources are all defined in the same way. `(resource-type/ensure)` or
`(resource-type/remove)`, a string name, and pairs of symbol keys and string
values. The `:keyword "string"` format is the way we idiomatically define
key-value pairs in Janet. You can't use commas.

Note that the `:owner` of the directory is a Janet keyword. This is a reference
which will be followed, and make the owner of the directory the same as that of
the file. References can refer to other references, and `gurp` is able to detect
unresolvable loops and dangling references.

Typically a reference takes the form
`role-name/resource-type/resource-name/resource-property`. Internally every
resource gets an ID constructed in that way, and you will see them in execution
output. But, the slash-separation could be confusing when referring to file
paths, so `gurp` converts slashes to underscores in the id resource property
name. You can use that pattern in your references, but to save you having to
even deal with that, you can, follow the above example and add a `:label`, then
refer to that.

We have dynamically constructed a value for the content of the file. `(string)`
concatenates its arguments, and `(this-host)` is a `gurp` builtin which expands
to the name you set in your `(host)` definition.

## Variables

Because it aims to be as stupid and unsophisticated as possible, `gurp` does not
have attribute hierarchies or any kind of inbuilt variable management. It's up
to you to use Janet. Use `(let)`, use `(def)`, use `(var)`, use prototyped
tables, structs, arrays, tuples, whatever works for you.

```clojure
# host.janet
(import "./vars")

(role "my-role"
  (each pkg (get vars/packages editors)
    (pkg/ensure (string "/ooce/editor/" pkg))))
```

```clojure
# vars.janet
(def packages
    {
      :editors [vim neovim helix]
      :languages [rust ruby33] } )
```

Isn't that more civilised than shoehorning weirdness into YAML and counting
indents?

Some resources have default values. We do this by means of
[Janet table prototypes](https://janet-lang.org/docs/prototypes.html). At the
moment, these are hardcoded into `gurp`, but by the time we're finished you will
also be able to supply your own.
