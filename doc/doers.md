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

The two most common actions are `ensure` and `remove`. The keys are resource-
and action-specific, and outlined below. The name is generally the same thing
the OS uses to identify that resource, so for a file it would be the path; for a
user, the username.

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

`:owner` and `:group` can be names or numeric IDs, but either way, quote them.
`mode` is a four-character octal string.

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

The difference is that files need some content. At the moment you have to
provide the file's contents inline. This may change at some point, but so far I
haven't needed to copy very large or binary files.

If you want to keep your file separate, Janet can read a file from local storage
with `(slurp)`, and you can also embed your content in the role file itself with
a `(def)` and reference that.

You can template files with `(template-out)`. This takes two arguments: the
first is a template, with variable keys denoted like `{{ this }}`. You also have
to provide a struct or table mapping those keys to values. For instance:

```clojure
(template-out "{{ prog }} is my new favourite {{ os }} tool"
              { :prog "gurp"
                :os "illumos" })
```

You can, of course, `(slurp)` the file off disk, and/or programmatically
generate your values. If your vars don't line up, `gurp` will error and tell you
why.

### User

User resources are mostly managed by shelling out to the `useradd(1m)`,
`usermod(1m)`, and `userdel(1m)` commands, so it shares their behaviour, and
might fail if trying to modify certain properties of a logged-in user.

Only the essentials are covered. The default shell is `/bin/zsh` and the default
`primary-group` is `staff`. Everything except `:passwowrd-hash` must be
specified. To unlock an account, use a hash of `NP`.

```clojure
(user/ensure "rdf"
             :gcos "My Real Name"
             :primary-group "sysadmin"
             :home-dir "/export/rob/rdf"
             :other-groups ["wheel"]
             :password-hash "s0mECr4zyHa$h"
             :shell "/bin/zsh")
```

The user is added to `:other-groups` when it is created, but `gurp` currently
lacks the ability to change that value on subsequent runs. There's an issue, so
it will get done at some point.

For `password-hash`, `gurp` has to manually manipulate `/etc/shadow`. There's no
other way to do it.

### Package

Package support is, for now at least, as basic as it can be. You can make sure a
package is installed or not installed with one of

```clojure
(pkg/ensure "ooce/developer/rust")
(pkg/remove "ooce/developer/go-124" )
```

`gurp` currently only supports ipkg packages, and does not provide for upgrades
or version pinning. You have to specify the package name as shown above; it's
the format you see if you run `pkg list -a`.

If you run `gurp` with `--noop`, `pkg(1)` will be executed, but with the `-n`
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
lines have a newline forced at the front, in case there wasn't already one at
the end of the file.

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

### Cron

Here's a fully explicit definition of a cron job.

```clojure
(cron/ensure "identifying-name"
             :hour "6,12"
             :minute "4"
             :day-of-month "*"
             :day-of-week "*"
             :month-of-year "*"
             :user "root"
             :command "/usr/bin/thing >/var/log/file")
```

If you omit any of the time fields, they will default to `"*"`. `:user` defaults
to `root`, and if you omit `:command`, you'll get an error. You can put numbers
or strings in there.

Like all other config management tools, `gurp` precedes managed lines in the
crontab with an identifying string. That string contains the resource ID which,
you may recall, includes the role, resource-type and identifying-name.

As illumos doesn't have the kind of `cron.d` support that some other OSes have,
`gurp` has to use the user's proper crontab, which it does by shelling out to
`/bin/crontab`. This gives you `crontab`'s standard value checking: `gurp`
doesn't check any values itself.

The doer doesn't do any kind of user or `cron.allow` management, so you'll have
to use other methods to make sure your user is allowed to run the job you
define.

To remove a cron job you already defined:

```clojure
(cron/remove "identifying-name")
```

There's currently no way to assert that a system-defined job does or does not
exist.

## Svc

`Svc` manages the state of SMF services, `Smf` is used to define them.

```clojure
(svc/ensure "svc:/vendor/category/servce:default"
             :state "online"
             :restarted-by ["/role/resource-type/name-or-label"]
             :reloaded-by ["/role/resource-type/name-or-label"])
```

Because `gurp` ends up shelling out to `svcs` and `svcadm`, the name can be any
valid FMRI.

`:state` can only be `online` or `disabled`. If you do not supply a state, it
defaults to `online`.

`:restarted-by` and `:reloaded-by` are optional arrays of resource IDs. If
`gurp` makes a change to any listed resource during its run, it will restart or
reload the service.

There is no `(svc/disable)`.

## Misc

There are certain tasks I used to manage with shell-script bodges. The `misc`
doer is where I turn them into proper, reliable code.

Currently the only thing the `misc` doer does is set the NFS domain. Note that
you don't give a resource name to this doer: it wouldn't make sense.

```clojure
(misc/ensure
             :nfs-domain "lan.id264.net")
```

There is no `(misc/remove)`.

## SMF

The `Smf` doer lets you define (limited) SMF services as Janet code. It supports
just the things I needed up to now.

```clojure
(smf/ensure "telegraf"
       :description "Run Telegraf agent"
       :fmri "sysdef/telegraf"
       :start-method {
         :exec "/bin/sleep 1200"
         :timeout 60
         :context {                                                                               
           :user "telegraf"
           :group "daemon"
           :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"}}
       :stop-method {
         :exec ":kill"
         :timeout 10 }
       :refresh-method {
         :exec ":kill -THAW"
         :timeout 60 })
```

If you don't supply a `:stop-method` you'll get a standard `:kill` that times
out after ten seconds. Start timeouts default to 60 seconds.

It isn't possible to have SMF tell you what manifest was imported, and even
comparing an export with the thing you just imported shows differences. So,
`gurp` generates an SMF manifest, writes it to disk, and will delete and
reimport a manifest if it sees a difference between that and the thing you
request. This will, of course, clobber any changes you've made.
