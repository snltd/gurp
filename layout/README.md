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
(gurp/role basenode
      :package "helix"
      :file {:name "basenode_file"
             :path "/tmp/basenode.txt"
             :content "some words"}
      :directory {:name "merp"
                  :path "/tmp/merp"
                  :mode "0755"})
```

Resources are all defined in the same way. `ensure` or `remove`, a string name,
and a table of options with symbol keys.

If you want to use variables, use `(var)` or `(def)`, and have them expanded at
compile time. You can put as much or as little actual Janet as you like in your
definition.

Resources have default values. These are defined in a top-level `defaults.janet`
which will eventually be supplied by the executable as a "default-default".

The `gurp/host` macro expands to a function `machine-config`, which iterates
over the roles, and merges all their resources into a single array.

You do not need a Janet runtime. The Rust part of `gurp` calls `machine-config`,
and exectues the Janet code in its built-in Janet interpreter.
