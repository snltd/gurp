> **This is part documentation, part thinking-out-loud, part
> README-driven-development. It might all change. Trust nothing.**

## Doers and Resources

Doers are the things that do the things. Not the best name, but neither are any
of the ones other people have come up with for similar components.

When a doer does its thing, it makes a resource which aligns with a resource
definiion.

Resource definitions look like Janet function calls. (Because that is what they
are.) Their format is

```clojure
(resource-type/action "resource-name"
  :key-1 "value-1"
  :key-2 "value-2")
```

There are two actions, `ensure` and `remove`. The keys are resource- and
action-specific, and outlined below. The name is generally the same thing the OS
uses to identify that resource, so for a file it would be the path; for a user,
the username.

## The Doers

### Directory

Directories are defined like this.

```clojure
(directory/ensure "/path/to/directory"
                  :mode "0750"
                  :owner "user-name"
                  :group "group-name")
```

At the moment `:owner` and `:group` must be strings: numeric IDs are not
supported. `mode` is a four-character octal string.

If you do not supply an `:owner` or `:group`, they will default to `root`.

Directories are created in a `mkdir -p` stylee, though only the named directory
will get the owner, group, and mode you specified. Ancestors will be owned by
whatever user `gurp` runs as, and created with its `umask`.

To make sure a directory does not exist,

```clojure
(directory/remove "/path/to/directory")
```

This will not remove any empty ancestors, but **will** remove the contents of
the directory.

### User

User resources are managed by shelling out to the `useradd(1m)`, `usermod(1m)`,
and `userdel(1m)` commands, so it shares their behaviour, and might fail if
trying to modify certain properties of a logged-in user.

Only the essentials are covered. The default shell is `/bin/zsh` and the default
`primary-group` is `staff`. Everything else must be specified.

```clojure
(user/ensure "rdf"
             :gcos "My Real Name"
             :primary-group "sysadmin"
             :home-dir "/export/rob/rdf"
             :other-groups ["wheel"]
             :shell "/bin/zsh")
```

### Package

Package support is, for now at least, as basic as it can be. You can make sure a
package is installed or not installed with one of

```clojure
(pkg/ensure "/ooce/developer/rust")
(pkg/remove "/ooce/developer/go-124" )
```

Gurp currently only supports ipkg packages, and does not provide for upgrades or
version pinning.

If you run gurp with `--noop`, `pkg(1)` will be executed, but with the `-n`
flag. Therefore it can cause a noop run to fail.
