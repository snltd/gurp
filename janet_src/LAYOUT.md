## Sample Machine Definition

This is a work-in-progress and subject to (possibly complete) change, but the
general principle is that you have a top-level `.janet` file which selects any
number of roles. Example:

```
(import ./roles/devtools)

(gurp/host "example"
  :roles [
    "devtools"
  ])
```

I might make the executable fill in the imports based on the `:roles`, but not
yet, because it's still all too much in flux.

Roles are defined by similar Janet files in `roles/`, and look like this (though
potentially subject to huge amounts of change):

```
(role "my-role"
      (pkg/ensure "helix")
      (file/ensure "config-file"
             :path "/etc/config.txt"
             :owner "rob"
             :content "config values")
      (directory/ensure "data-dir"
                  :path "/data"
                  :owner :/my-role/file/config-file/owner
                  :group "engineering"
                  :mode "0775"))
```

Resources are all defined in the same way. `ensure` or `remove`, a string name,
and pairs of symbol keys and string values.

Note that the `:owner` of the directory is a Janet keyword. This is a reference
which will be followed, and make the owner of the directory the same as that of
the file. References can refer to other references, and `gurp` is able to
detect unresolvable loops and dangling references.

If you want to use variables, use `(var)` or `(def)`, and have them expanded at
compile time. You can put as much or as little actual Janet as you like in your
definition. Remember, you have a full, powerful programming language at your
disposal.

Resources have default values. These are defined in a top-level `defaults.janet`
which will eventually be supplied by the executable as a "default-default".

The `gurp/host` macro expands to a function `machine-config`, which iterates
over the roles, and merges all their resources into a single array.

You do not need a Janet runtime. The Rust part of `gurp` calls `machine-config`,
and exectues the Janet code in its built-in Janet interpreter.
