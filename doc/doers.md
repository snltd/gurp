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

All doers do the bare minimum needed to build my systems. If you want more, open
an issue or a PR.

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

### File

Files are mostly created like directories:

```clojure
(file/ensure "/path/to/file"
             :mode "0750"
             :owner "user-name"
             :group "group-name"
             :content "some content")
```

But the big difference is that files need some content. You can specify literal
content with `:content`, or you can use `:from`, which tells `gurp` to copy a
file.

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

### File-line

This makes sure that the given lines are, or are not, in the given file. If the
file does not exist, the doer will fail, so you may have to manage the file with
a `(file)` resource. This seems more efficient than duplicating all the `(file)`
functionality here. Files are created before lines are managed, so the
dependency is implicit.

Like all doers, `(file-line)` is very stupid. If the line does not exist it will
be appended to the file. If it does, it's left where it is. Removing a line will
add a newline to the end of the file, if there isn't one already, and appended
lines have a newline forced at the front, in case there wasn't one at the end of
the file.

You can only manage one line per resource, because if we do add things like
`:line-number`, or `:before` or whatever, it'll be a lot more straightforward.

```clojure
(file-line/ensure "/path/to/file"
                  :line "this is the line I want")
```

```clojure
(file-line/remove "/path/to/file"
                  :line "this is the first line I do not want")
```
