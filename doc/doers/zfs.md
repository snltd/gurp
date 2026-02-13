# zfs

Create, destroy, and modify properties of ZFS filesystems.

## Resource Name

ZFS dataset name (`:string`)

## zfs/ensure

```janet
(zfs/ensure "tank/example/filesystem"
            :label "zfs-example-1"
            :properties {:compression "gzip9"
                         :mountpoint "/example/mountpoint"
                         :dedup true
                         :devices false})
```

```janet
(zfs/ensure "tank/example/volume"
            :size "10G"
            :label "example-zfs-vol")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:properties` | `struct` | ZFS properties (:keyword) paired with desired value (:string) | `{:mountpoint "none"}` |
| `:size` | `string` | If specified, creates a ZFS volume of given size (e.g. '10G') |  |

## zfs/remove

```janet
(zfs/remove "tank/old/filesystem")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- Gurp does not check parameters are valid, so if you get them wrong the first you'll know about it is when you get an error from `zfs(8)`.
- Gurp cannot change the size of an extant volume.
- zfs/destroy is recursive, and will remove all child filesystems and snapshots without asking or telling.
