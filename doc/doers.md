# Doers and Resources

Doers are the things that do the things. Not the best name, but neither are any
of the ones other people have come up with for similar components.

When a doer does its thing, it makes a resource which aligns with a resource
spec.

Resource specs look like Janet function calls. (Because that is what they are.)
Their format is

```janet
(resource-type/action "resource-name"
                      :key-1 "value-1"
                      :key-2 "value-2")
```

Actions are `ensure` and `remove`. The keys are resource- and action-specific,
and outlined below. The name is generally the same thing the OS uses to identify
that resource, so for a file it would be the path; for a user, the username.

All doers do the bare minimum needed to build my systems. If you want more, open
an issue or a PR.

## Ordering

Doers are executed in the following order:

- [`(publisher/ensure)`]((publisher/ensure))

## `apk`

### `(apk/ensure)`

```janet
(apk/ensure "rust")
```

The `apk` doer adds and removes packages in an Alpine LX zone.

| Key  | Type   | Description  | Default | Mandatory |
| ---- | ------ | ------------ | ------- | --------- |
| Name | string | Package name |         | yes       |

The package database is refreshed prior to install.

You cannot install or pin specific versions.

### `(apk/remove)`

```janet
(apk/remove "go" )
```

| Key  | Type   | Description  | Default | Mandatory |
| ---- | ------ | ------------ | ------- | --------- |
| Name | string | Package name |         | yes       |

## `cron`

### `(cron/ensure)`

```janet
(cron/ensure "identifying-name"
             :hour "6,12"
             :minute "4"
             :day-of-month "*"
             :day-of-week "1-5"
             :month-of-year "*"
             :user "batch"
             :command "/usr/bin/thing >/var/log/file")
```

| Key             | Type           | Description                                   | Default | Mandatory |
| --------------- | -------------- | --------------------------------------------- | ------- | --------- |
| Name            | string         | Some name to identify the job                 |         | yes       |
| `command`       | string, number | Command to run                                |         | yes       |
| `day-of-month`  | string, number | Day(s) of month on which job runs             | `*`     |           |
| `day-of-week`   | string, number | Day(s) of week on which job runs. 0 is Sunday | `*`     |           |
| `hour`          | string, number | Hour(s) at which job runs                     | `*`     |           |
| `minute`        | string, number | Minute(s) at which job runs                   | `*`     |           |
| `month-of-year` | string, number | Month(s) of year in which job runs            | `*`     |           |
| `user`          | string, number | User job runs as                              | `root`  |           |

`hour`, `minute` etc can take any valid illumos cron value, so `5,10,15` or
`*/5` are fine.

Like all other config management tools, `gurp` precedes managed lines in the
crontab with an identifying string. That string contains the resource ID which,
includes the role, resource-type and identifying-name.

As illumos doesn't have the kind of `cron.d` support that some other OSes have,
`gurp` has to use the user's proper crontab, which it does by shelling out to
`/bin/crontab`. This gives you `crontab`'s standard value checking: `gurp`
doesn't check any values itself.

The doer doesn't do any kind of user or `cron.allow` management, so you'll have
to use other methods to make sure your user is allowed to run the job you
define.

### `(cron/remove)`

```janet
(cron/remove "identifying-name")
```

| Key  | Type   | Description                   | Default | Mandatory |
| ---- | ------ | ----------------------------- | ------- | --------- |
| Name | string | Some name to identify the job |         | yes       |

This doer has no way to assert that a system-defined job does or does not exist.

## `directory`

### `(directory/ensure)`

```janet
(directory/ensure "/path/to/directory"
                  :mode "0750"
                  :owner "user-name"
                  :group "group-name")
```

| Key     | Type           | Description                        | Default | Mandatory |
| ------- | -------------- | ---------------------------------- | ------- | --------- |
| Name    | string         | Fully-qualified path               |         | yes       |
| `group` | string, number | Can be a group name or numeric GID | `root`  |           |
| `mode`  | string         | Four-character octal string        | `0755`  |           |
| `user`  | string, number | Can be a username or numeric UID   | `root`  |           |

Directories are created in a `mkdir -p` style, though only the named directory
will get the owner, group, and mode you specified. Ancestors will be owned by
whatever user `gurp` runs as, and created with its `umask`.

### (directory/remove)

```janet
(directory/remove "/path/to/directory")
```

| Key  | Type   | Description          | Default | Mandatory |
| ---- | ------ | -------------------- | ------- | --------- |
| Name | string | Fully-qualified path |         | yes       |

This will not remove any empty ancestors, but **will** remove the contents of
the directory.

## `file`

### `(file/ensure)`

```janet
(file/ensure "/path/to/file"
             :mode "0750"
             :owner "user-name"
             :group "group-name"
             :content "some content")
```

| Key              | Type           | Description                                                    | Default | Mandatory |
| ---------------- | -------------- | -------------------------------------------------------------- | ------- | --------- |
| Name             | string         | Fully-qualified path                                           |         | yes       |
| `content`        | string         | Literal file content                                           |         | yes [*]   |
| `from`           | string         | Path to a file which will be copied in                         |         | yes [*]   |
| `group`          | string, number | Can be a group name or numeric GID                             | `root`  |           |
| `ignore-pattern` | string         | When diffing files, Gurp will ignore lines matching this regex |         |           |
| `mode`           | string         | Four-character octal string                                    | `0755`  |           |
| `user`           | string, number | Can be a username or numeric UID                               | `root`  |           |

[*] You must supply exactly one of `:content` or `:from`.

The `(template-out)` and `(indoc)` helpers are useful when specifying
`:content`.

`:from` takes a fully-qualified or relative path. If you use the latter, Gurp
assumes the file is in a `files/` directory at the same level as the directory
holding the file being parsed.

### `(file/remove)`

```janet
(file/remove "/path/to/directory")
```

| Key  | Type   | Description          | Default | Mandatory |
| ---- | ------ | -------------------- | ------- | --------- |
| Name | string | Fully-qualified path |         | yes       |

## `file-line`

### `(file-line/ensure)`

```janet
(file-line/ensure "/path/to/file"
                  :line "The line I want")
```

If `The line I want` is a complete line anywhere in `/path/to/file`, no action
is taken. If it is not, the line is appended to the end of the file. Modified
files always have a trailing newline.

```janet
(file-line/ensure "/path/to/file"
                  :replace "this" :with "that"
                  :apply-to "first")
```

This will look at each line in turn, and the first time it sees `this` it will
replace it with `that`. The matching and replacing is done with a Rust regex.

| Key         | Type   | Description                                                                | Default | Mandatory |
| ----------- | ------ | -------------------------------------------------------------------------- | ------- | --------- |
| Name        | string | File path                                                                  |         | yes       |
| `:apply-to` | string | When replacing, which matches to replace. Can be `first`, `last`, or `all` | `all`   | yes [*]   |
| `:line`     | string | Line which must exist                                                      |         | yes [*]   |
| `:replace`  | string | Pattern to replace                                                         |         | yes [*]   |
| `:with`     | string | String with which to replace                                               |         | yes [*]   |

[*] You must supply exactly one of `:line` or `:replace`. `:replace` must be
paired with `:with`.

If the file does not exist, the doer will fail, so you may have to manage the
file with a `(file)` resource. Files are created before lines are managed.

### `(file-line/remove)`

```janet
(file-line/remove "/path/to/file" :pattern "remove these lines")
```

```janet
(file-line/remove "/path/to/file"
                  :pattern "ip-address="
                  :match "starts-with"
                  :apply-to "last")
```

```janet
(file-line/remove "/path/to/file"
                  :pattern "^ip-address=.*\.168\..*/32$"
                  :match "regex"
                  :apply-to "all")
```

| Key         | Type   | Description                                                                                 | Default | Mandatory |
| ----------- | ------ | ------------------------------------------------------------------------------------------- | ------- | --------- |
| Name        | string | File path                                                                                   |         | yes       |
| `:apply-to` | string | When replacing, which matches to replace. Can be `first`, `last`, or `all`                  | `all`   |           |
| `:match`    | string | How to match `:pattern`. Can be `exact`, `starts-with`, `ends-with`, `contains`, or `regex` | `exact` |           |
| `:pattern`  | string | Pattern used to identify unwanted line                                                      |         | yes [*]   |

If `(file-line/remove)` removes a line, it will always add a newline to the end
of the file, if there isn't one already.

## `gem`

Manages Ruby gems.

### `(gem/ensure)`

```janet
(gem/ensure "webscale")
```

```janet
(gem/ensure "wavefront-cli"
             :source "http://my.gem.repo.com"
             :gem-path "/opt/local/bin/gem"
             :version "10.0.1")
```

| Key         | Type   | Description                    | Default             | Mandatory |
| ----------- | ------ | ------------------------------ | ------------------- | --------- |
| Name        | string | The link to remove             |                     | yes       |
| `:gem-path` | string | Install with this `gem` binary | `/opt/ooce/bin/gem` |           |
| `:source`   | string | Gem repo to use                | RubyGems            |           |
| `:version`  | string | Version to install             |                     |           |

### `(gem/remove)`

```janet
(gem/remove "nokogiri")
```

| Key         | Type   | Description                   | Default             | Mandatory |
| ----------- | ------ | ----------------------------- | ------------------- | --------- |
| Name        | string | The link to remove            |                     | yes       |
| `:gem-path` | string | Remove with this `gem` binary | `/opt/ooce/bin/gem` |           |
| `:version`  | string | Version to remove             | all installed       |           |

## `group`

### `(group/ensure)`

```janet
(group/ensure "ai-users")
```

| Key   | Type   | Description | Default | Mandatory |
| ----- | ------ | ----------- | ------- | --------- |
| Name  | string | Group name  |         | yes       |
| `gid` | number | Group ID    |         | yes       |

### `(group/remove)`

```janet
(group/remove "real-people" )
```

| Key  | Type   | Description | Default | Mandatory |
| ---- | ------ | ----------- | ------- | --------- |
| Name | string | Group name  |         | yes       |

## `misc`

There are certain tasks I used to manage with shell-script bodges. The `misc`
doer is where I turn them into proper, reliable code. They are all what I would
consider "primitive" operations. That is, they cannot be accomplished through a
combination of other doers.

### `(misc/ensure)`

```janet
(misc/ensure :nfs-domain "lan.id264.net")
```

```janet
(misc/ensure :enable-smb "rob")
```

```janet
(misc/ensure :scheduler "FSS")
```

| Key           | Type   | Description                                   | Default | Mandatory |
| ------------- | ------ | --------------------------------------------- | ------- | --------- |
| Name          | string | Some name to identify the job                 |         | yes       |
| `:nfs-domain` | string | NFS domain                                    |         |           |
| `:enable-smb` | string | Use for whom SMB shares should be enabled     |         |           |
| `:scheduler`  | string | Scheduler class. Only applies to global zones |         |           |

### `(misc/remove)`

There is no `(misc/remove)`.

## `pkg`

### `(pkg/ensure)`

```janet
(pkg/ensure "ooce/developer/rust")
```

| Key  | Type   | Description  | Default | Mandatory |
| ---- | ------ | ------------ | ------- | --------- |
| Name | string | Package name |         | yes       |

Packages must be specified in the format shown above: it's the format you see if
you run `pkg list -a`.

You cannot currently install specific versions, and there is no support for
mediators.

If you run `gurp` with `--noop`, `pkg(1)` will be executed, but with the `-n`
flag. Therefore it can cause a noop run to fail.

### `(pkg/remove)`

```janet
(pkg/remove "ooce/developer/go-124" )
```

| Key  | Type   | Description  | Default | Mandatory |
| ---- | ------ | ------------ | ------- | --------- |
| Name | string | Package name |         | yes       |

## `pkgin`

### `(pkgin/ensure)`

```janet
(pkgin/ensure "rust")
```

The `pkgin` doer lets you add and install packages in `pkgsrc` branded zones.

| Key  | Type   | Description  | Default | Mandatory |
| ---- | ------ | ------------ | ------- | --------- |
| Name | string | Package name |         | yes       |

You cannot currently install specific versions.

### (pkgin/remove)

```janet
(pkgin/remove "go" )
```

| Key  | Type   | Description  | Default | Mandatory |
| ---- | ------ | ------------ | ------- | --------- |
| Name | string | Package name |         | yes       |

## `publisher`

### `(publisher/ensure)`

```janet
(publisher/ensure "sysdef" :uri "http://pkg.lan.id264.net/")
```

| Key    | Type   | Description      | Default | Mandatory |
| ------ | ------ | ---------------- | ------- | --------- |
| Name   | string | Publisher name   |         | yes       |
| `:uri` | string | URI of publisher |         | yes       |

The `publisher` doer only manages origins. You can't configure mirrors.

### `(publisher/remove )`

```janet
(publisher/remove "sysdef")
```

| Key  | Type   | Description    | Default | Mandatory |
| ---- | ------ | -------------- | ------- | --------- |
| Name | string | Publisher name |         | yes       |

## `smf`

The `smf` doer (not to be confused with `svc`) lets you define SMF services as
Janet code.

### `(smf/ensure)`

```janet
(smf/ensure "telegraf"
            :description "Run Telegraf agent"
            :fmri "sysdef/telegraf"
            (exec-method "start"
                         :exec "/bin/sleep 1200"
                         :timeout 60
                         :user "telegraf"
                         :group "daemon"
                         :privileges ["basic" "file_dac_search" "sys_admin"
                                      "proc_owner" "proc_zone"])
            (exec-method "refresh"
                         :exec ":kill -THAW"
                         :timeout 60)
            :properties {:restarter/contract "fixed"
                         :restarter/count 10
                         :restarter/delay 10}
            :environment {:LC_CTYPE "en_US.UTF-8"})
```

```janet
(smf/ensure startup-svc
            :fmri "sysdef/application/service-setup"
            :description "transient service"
            :duration "transient"
            (smf-method "start" :exec "some-method-or-other"))
```

| Key                | Type                    | Description                                        | Default | Mandatory |
| ------------------ | ----------------------- | -------------------------------------------------- | ------- | --------- |
| Name               | string                  | The service name                                   |         | yes       |
| `:default-enabled` | bool                    | Start the service when the manifest installs       | true    |           |
| `:description`     | string                  | What the service does                              |         |           |
| `:duration`        | string                  | Use this to specify `transient` or `wait` services |         |           |
| `:exec-method`     | function                | See below                                          |         |           |
| `:properties`      | struct<keyword, string> | Create/set properties                              |         |           |
| `:property-groups` | list<string>            | Create the given property groups                   |         |           |
| `:single-instance` | bool                    | Whether this is a single-instance service          | true    |           |

#### `(exec-method)`

The `(exec-method)` function is used to define the methods and contexts which
start and stop the service. Its spec is a flat structure: Gurp puts things into
the correct nested structs.

| Key            | Type                    | Description                                                | Default             | Mandatory |
| -------------- | ----------------------- | ---------------------------------------------------------- | ------------------- | --------- |
| Name           | string                  | What the method does. `start`, `stop`, `reload`, `refresh` |                     | yes       |
| `:environment` | struct<keyword, string> | Environment variables set in method context                |                     |           |
| `:exec`        | string                  | Method or command to execute                               |                     | yes       |
| `:group`       | string                  | Group method runs as                                       |                     |           |
| `:privileges`  | list<string>            | Privileges method is invoked with. Use `!` to remove them  |                     |           |
| `:timeout`     | number                  | Seconds until method times out                             | 60, but for `stop`, | yes       |
| `:user`        | string                  | User method runs as                                        |                     |           |

If you don't supply a `:stop-method` youget a standard `:kill` that times out
after ten seconds. Start timeouts default to 60 seconds.

It isn't possible to have SMF tell you what manifest was imported, and even
comparing an export with the thing you just imported shows differences. So,
`gurp` generates an SMF manifest, writes it to disk, and will delete and
reimport a manifest if it sees a difference between that and the thing you
request. This will, of course, clobber any changes you've made.

### `(smf/remove)`

This stops the service and deletes it from the SMF registry.

```janet
(smf/remove "ex-service")
```

| Key  | Type   | Description      | Default | Mandatory |
| ---- | ------ | ---------------- | ------- | --------- |
| Name | string | The service name |         | yes       |

## `svc`

### `(svc/ensure)`

`svc` manages the state of SMF services, `smf` is used to define them.

```janet
(svc/ensure "svc:/vendor/category/servce:default"
             :state "online"
             :restarted-by ["/role/resource-type/name-or-label"]
             :reloaded-by ["/role/resource-type/name-or-label"])
```

| Key            | Type         | Description                                                                      | Default  | Mandatory |
| -------------- | ------------ | -------------------------------------------------------------------------------- | -------- | --------- |
| Name           | string       | Any valid FMRI                                                                   |          | yes       |
| `reloaded-by`  | list<string> | Gurp identifiers of resource which, when changed, will trigger a service reload  |          |           |
| `restarted-by` | list<string> | Gurp identifiers of resource which, when changed, will trigger a service restart |          |           |
| `;state`       | string       | Service state as shown by `svcs`                                                 | `online` |           |

Because `gurp` ends up shelling out to `svcs` and `svcadm`, the name can be any
valid FMRI.

### `(svc/remove)`

There is no `(svc/remove)`.

## `symlink`

### `(symlink/ensure)`

```janet
(symlink/ensure "/my/link" :source "/my/file")
(symlink/remove "/my/other/link")
```

| Key       | Type   | Description             | Default | Mandatory |
| --------- | ------ | ----------------------- | ------- | --------- |
| Name      | string | The link to create      |         | yes       |
| `:source` | string | What the link points to |         | yes       |

If the `:source` doesn't exist, you get an error. Files are ensured before
links, so you can make a file and link to it. If the link exists and points to
the wrong file, it will be removed and re-created, and if it exists but is not a
link, that's an error.

Hard links are not supported.

### `(symlink/remove )`

```janet
(symlink/remove "/unwanted/link")
```

| Key  | Type   | Description        | Default | Mandatory |
| ---- | ------ | ------------------ | ------- | --------- |
| Name | string | The link to remove |         | yes       |

## `user`

### `(user/ensure)`

```janet
(user/ensure "rdf"
             :gecos "My Real Name"
             :primary-group "sysadmin"
             :home-dir "/export/rob/rdf"
             :other-groups ["wheel"]
             :password-hash "s0mECr4zyHa$h"
             :shell "/bin/zsh")
```

| Key             | Type           | Description                                                     | Default | Mandatory |
| --------------- | -------------- | --------------------------------------------------------------- | ------- | --------- |
| Name            | string         | Username                                                        |         | yes       |
| `gecos`         | string         | User's name                                                     |         | yes       |
| `home-dir`      | string         | Fully qualified path to home directory                          |         | yes       |
| `other-groups`  | string         | User will be added to these in `/etc/group`                     |         | yes       |
| `password-hash` | string         | Will be set as second field in `/etc/shadow`                    |         |           |
| `primary-group` | string, number | Group name or numeric GID of the group defined in `/etc/passwd` | `root`  | yes       |
| `shell`         | string         | Fully qualified path to user's shell                            |         | yes       |
| `uid`           | number         | UID                                                             |         | yes       |

User resources are mostly managed by shelling out to the `useradd(1m)`,
`usermod(1m)`, and `userdel(1m)` commands, so the doer shares their behaviour,
and might fail if you try to modiy properties of a logged-in user.

To unlock an account, use a hash of `NP`.

### `(user/remove)`

```janet
(user/remove "username")
```

| Key  | Type   | Description | Default | Mandatory |
| ---- | ------ | ----------- | ------- | --------- |
| Name | string | Username    |         | yes       |

## `zfs`

### `(zfs/ensure)`

```janet
(zfs/ensure "tank/filesystem"
            :properties {:mountpoint "/data/u01"
                         :compression "gzip-9"
                         :setuid "off"})
```

```janet
(zfs/ensure "tank/volume"
            :size "100G")
```

| Key           | Type                   | Description                      | Default | Mandatory |
| ------------- | ---------------------- | -------------------------------- | ------- | --------- |
| Name          | string                 | Dataset to create                |         | yes       |
| `:properties` | struct<string, string> | Any valid ZFS property and value |         |           |
| `:size`       | string                 | Create a ZFS volume of this size |         |           |

Gurp does not check parameters are valid, so if you get them wrong the first
you'll know about it is when you get an error from `zfs(8)`.

Gurp cannot change the size of an extant volume.

### `(zfs/remove)`

```janet
(zfs/remove "tank/old-dataset")
```

| Key  | Type   | Description       | Default | Mandatory |
| ---- | ------ | ----------------- | ------- | --------- |
| Name | string | Dataset to remove |         | yes       |

Remove is done with `-R`, so it takes all snapshots with it.

## `svcprop`

### `(svcprop/ensure)`

```janet
(svcprop/ensure "my/cool/service:default")
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"})
```

| Key                | Type                    | Description                               | Default | Mandatory |
| ------------------ | ----------------------- | ----------------------------------------- | ------- | --------- |
| Name               | string                  | FMRI of service                           |         | yes       |
| `:properties`      | struct<keyword, string> | Create or set these properties            |         | yes       |
| `:property-groups` | struct<keyword, string> | Create property group `key` of type `val` |         |           |

`gurp` will infer and add the property types.

### `(svcprop/remove)`

```janet
(svcprop/remove "my/other/service:default")
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"})
```

| Key                | Type         | Description                  | Default | Mandatory |
| ------------------ | ------------ | ---------------------------- | ------- | --------- |
| Name               | string       | FMRI of service              |         | yes       |
| `:properties`      | list<string> | Remove these properties      |         | yes       |
| `:property-groups` | list<string> | Remove these property groups |         |           |

## `zone`

### `(zone/ensure)`

```janet
(zone/ensure "serv-www-proxy"
             :brand "lipkg"
             :clone-from gold-zone
             :capped-memory {:physical "300m" :swap "300m"}
             (zone-fs "/home" :special "/export/home")
             (zone-network "wwwpx_net0"
                           :allowed-address "192.168.1.25/24")
                           :defrouter "192.168.1.1")
             :dns {:domain "lan.id264.net"
                   :nameservers ["192.168.1.1" "192.168.1.53"]}
             :bootstrap-from (pathcat gurp-dir "zone-www-proxy.janet"))
```

```janet
(zone/ensure "serv-grafana"
             :brand "lx"
             :lx-image "alpine"
             :recreate 1
             :final-state "reboot"
             (zone-attr "kernel-version" :value "4.4")
             (zone-network "wwwpx_net0"
                           :allowed-address "192.168.1.25/24")
                           :defrouter "192.168.1.1")
             :dns globals/zone-dns
             :datasets ["tank/zone/grafana")]
             :bootstrap-from (pathcat gurp-dir "zone-grafana.janet"))
```

| Key                   | Type                    | Description                                                                                                                    | Default              | Mandatory |
| --------------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------- | --------- |
| Name                  | string                  | Zone name                                                                                                                      |                      | yes       |
| `:autoboot`           | bool                    | Boot the zone on system boot                                                                                                   | true                 |           |
| `:boot-after-install` | bool                    | Boot the zone once it is installed                                                                                             | true                 |           |
| `:bootstrap-from`     | string                  | Copy gurp into the zone, and apply the given file, relative to zone root                                                       |                      |           |
| `:brand`              | string                  | Zone brand. One of `lipkg`, `ipkg`, `sparse`, `lx`                                                                             |                      |           |
| `:capped-memory`      | struct<keyword, string> | Set memory cap. Keys must be `:physical` and `:swap`                                                                           |                      |           |
| `:clone-from`         | string                  | Instead of installing, clone from the given zone, and be halted                                                                |                      |           |
| `:copy-in`            | struct<keyword, string> | Copy files into the zone. Key is source, value is dest, relative to zone root. Unqualified src is assumed to be in `../files/` |                      |           |
| `:datasets`           | list<string>            | ZFS datasets to be delegated to zone                                                                                           |                      |           |
| `:dns`                | struct                  | DNS config of the form `:domain "string" :nameservers list<string>"                                                            |                      |           |
| `:exec-in`            | list<string>            | Runs the given commands in the zone after booting                                                                              |                      |           |
| `:final-state`        | string                  | Put the zone in the given state. Can be `installed`, `ready` or `reboot`                                                       |                      |           |
| `:lx-image`           | string                  | Install an `lx` braned zone with this image                                                                                    |                      |           |
| `:recreate`           | number                  | 1-in-n chance the zone will be destroyed and recreated                                                                         | 0                    |           |
| `(zone-attr)`         | function                | See below                                                                                                                      |                      |           |
| `(zone-fs)`           | function                | See below                                                                                                                      |                      |           |
| `(zone-net)`          | function                | See below                                                                                                                      |                      |           |
| `:zonepath`           | string                  | Path to zone root                                                                                                              | `/zones/<zone-name>` |           |
| `(zone-rctl)`         | function                | See below                                                                                                                      |                      |           |

#### `(zone-attr)`

| Key      | Type                 | Description       | Default                  | Mandatory |
| -------- | -------------------- | ----------------- | ------------------------ | --------- |
| Name     | string               | Attribute name    |                          | yes       |
| `:type`  | string               | Type of attribute | inferred from Janet type |           |
| `:value` | string, number, bool | Attribute value   |                          | yes       |

#### `(zone-fs)`

| Key        | Type   | Description            | Default | Mandatory |
| ---------- | ------ | ---------------------- | ------- | --------- |
| Name       | string | Zone mountpoint        |         | yes       |
| `:special` | string | Global zone mountpoint |         | yes       |
| `:type`    | string | Type of mount          | `lofs`  |           |

#### `(zone-net)`

| Key                | Type   | Description             | Default | Mandatory |
| ------------------ | ------ | ----------------------- | ------- | --------- |
| Name               | string | Global zone VNIC name   |         | yes       |
| `:allowed-address` | string | IP of zone              |         | yes       |
| `:defrouter`       | string | IP of default router    |         |           |
| `:mac-address`     | string | MAC of zone VNIC        | `auto`  |           |
| `:physical`        | string | Underlying physical NIC | `auto`  |           |

#### `(zone-rctl)`

| Key      | Type   | Description    | Default | Mandatory |
| -------- | ------ | -------------- | ------- | --------- |
| Name     | string | RCTL name      |         | yes       |
| `:priv`  | number | RCTL privilege |         | yes       |
| `:value` | number | RCTL value     |         | yes       |

The doer cannot modify an existing zone.

`kvm`, `bhyve` and `illumos` zones are not currently supported.

**Notes on `:recreate`**. This must be an integer, and it is the `n:1` odds of a
zone being destroyed and recreated. So, `0` means "never recreate this zone",
and `1` means "recreate this zone on every run". `2` You can set the number as
high as you like, so if you run `gurp` every 15 minutes and want your zone
rebuilt from scratch about once a week, you'd set it to `672`. If you don't set
it, it defaults to `0`.
