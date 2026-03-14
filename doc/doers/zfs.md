# zfs

Create, destroy, and modify properties of ZFS filesystems.

## Resource Name

ZFS dataset name (`:string`)

## zfs/ensure

```janet
(zfs/ensure "rpool/example/filesystem"
            :label "zfs-example-1"
            :properties {:compression "gzip-9"
                         :mountpoint "/example/mountpoint"
                         :dedup true
                         :devices false})
```

```janet
(zfs/ensure "rpool/example/volume"
            :size "10G"
            :label "example-zfs-vol")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:properties` | `struct` | ZFS properties (:keyword) paired with desired value (:string) |  |
| `:size` | `string` | If specified, creates a ZFS volume of given size (e.g. '10G') |  |

## zfs/remove

```janet
(zfs/remove "rpool/old/filesystem")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- Gurp does not check parameters are valid, so if you get them wrong the first you'll know about it is when you get an error from `zfs(8)`.
- If you do not set a mountpoint for a filesystem, Gurp will force it to 'none'.
- Gurp cannot change the size of an extant volume.
- zfs/destroy is recursive, and will remove all child filesystems and snapshots without asking or telling.
