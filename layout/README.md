## Sample Machine Definition

This is a work-in-progress and subject to (possibly complete) change, but the
general principle is that you have a top-level `.janet` file which selects any
number of roles. Example:

```
(import ./lib/helpers)
(import ./roles/devtools)

(helpers/host "example"
  :roles [
    "devtools"
  ])
```

Roles are defined by similar Janet files in `roles/`, and look like this:

```
(use ../lib/helpers)

(role role
      :packages [(ensure "janet")
                 (ensure "rust" :version "latest")
                 (remove "go")]
      :users [(ensure "rob"
                      :uid 264
                      :gid 14
                      :dir "/home/rob")]
      :files [(ensure "sample"
                      :path "/tmp/merp/merp.txt"
                      :source "templates/merp.tmpl"
                      :vars {:var-1 "string 1"
                             :var-2 :user/rob/name})]
      :directories [(ensure "merp"
                            :path "/tmp/merp"
                            :owner :user/rob/uid
                            :group :user/rob/group
                            :mode "0755")
                    (ensure "gajerp"
                            :path "/tmp/gajerp"
                            :owner :dir/merp/owner
                            :group "root"
                            :mode "0775")])
```

Resources are all defined in the same way. `ensure` or `remove`, a string name,
and a table of options with symbol keys.

The `helpers/host` macro expands to a function `machine-config`, which iterates
over the roles, and merges all their resources into a single table.

You do not need a Janet runtime. The Rust part of `gurp` calls `machine-config`,
and exectues the Janet code in its built-in Janet interpreter.

Note that you can reference properties of another object using a path-like
symbol. When the Rust part of `gurp` process the example above, it will resolve
`:user/rob/uid` to `264`.
