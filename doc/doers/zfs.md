# zfs

Create, destroy, and modify properties of ZFS filesystems.

## Resource Name

ZFS dataset name (`:string`)

## zfs/ensure

```janet
(zfs/ensure "tank/example/volume"
            :size "10G"
            :label "example-zfs-vol")
```

```janet
(zfs/ensure "tank/example/filesystem"
            :label "zfs-example-1"
            :properties {:compression "gzip9"
                         :mountpoint "/example/mountpoint"
                         :dedup true
                         :devices false})
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:properties` | `struct` | ZFS properties (:keyword) paired with desired value (:string) | `{:mountpoint: "none"}` |
| `:size` | `string` | If specified, creates a ZFS volume of given size (e.g. '10G') |  |

## zfs/remove

```janet
(zfs/remove "tank/old/filesystem")
```

### Mandatory Properties

None

### Optional Properties

None

