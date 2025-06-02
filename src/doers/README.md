**This is part documentation, part thinking-out-loud, part
README-driven-development. It might all change. Trust nothing.**

## Doers and Resources

Doers are the things that do the things. Not the best name, but neither are the
ones other people have come up with for similar components. When a doer does its
thing, it makes a resource which aligns with a resource definiion.

Resource definitions look like Janet function calls. (Because that is what they
are.) Their format is

```clojure
(resource-type/action "resource-name"
  :key-1 "value-1"
  :key-2 "value-2")
```

There are two actions, `ensure` and `remove`. The keys are resource- and
action-specific, and outlined below. The name is for the user's convenience, and
for building references.

Resource properties can have default values. At the moment only hardcoded values
are supported, but eventually you will be able to add your own.

Resources can refer to other resources through a Janet keyword of the form
`role/resource-type/resource-name/property`.

```clojure
(role "thing-maker"
  (thing/ensure "first-thing"
    :owner "mr_thing")

  (thing/ensure "second-thing"
    :owner :thing-maker/thing/first-thing/owner))
```

The two `thing`s will both belong to `mr_thing`. Gurp can detect unresolved and
circular references. You can only refer to gurp-managed resources.

### Directory

Directories are defined like this.

```clojure
(directory/ensure "some-name"
  :mode "0750"
  :owner "user-name"
  :group "group-name"
  :recurse true)
```

At the moment `:owner` and `:group` must be strings: numeric IDs are not
supported. `mode` is a four-character octal string.

`:recurse` tells gurp to behave like `mkdir -p`. Any additional directories will
be created with the default user and umask of the gurp process.

To make sure a directory does not exist,

```clojure
(directory/remove "some-name"
  :recurse true)
```

In this case `:recurse` means "remove any files inside the directory". If it is
`false`, the directory will only be removed if it is empty.

gurp has hardcoded defaults of `root` for `:owner` and `:group`.

### Package

Package support is, for now at least, as basic as it can be. You can make sure a
package is installed or not installed with one of

```clojure
(package/ensure "/ooce/developer/rust")

(package/remove "/ooce/developer/go-124" )
```

Gurp currently only supports ipkg packages, and does not provide for upgrades or
version pinning.

If you run gurp with `--noop`, `pkg(1)` will be executed, but with the `-n`
flag. Therefore it can cause a noop run to fail.

### Users

User resources are created by shelling out to the `useradd(1m)` command. Only
the essentials are covered by keywords, but there's a mechanism to drop in
arbitrary options.

```clojure
(user/ensure "my-name"
  :username "rdf"
  :gcos "My Real Name"
  :primary-group "sysadmin"
  :home-dir "/export/rob/rdf"
  :other-groups ["wheel"]
  :shell "/bin/zsh"
  :useradd-options ["-z"])
```
