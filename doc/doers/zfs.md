# zfs

Create, destroy, and modify properties of ZFS filesystems.

## zfs/ensure

```janet
(zfs/ensure "tank/export/test-vol"
            :size "10G"
            :label "test-zfs-vol"
            :properties {:devices "off"})
```

```janet
(zfs/ensure (zfscat "tank" "export" "test")
            :label "test-zfs"
            :properties {:compression "gzip9"
                         :devices "off"})
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| properties | `struct` | ZFS properties (:keyword) paired with desired value (:string) | <struct 0x000001E44038> |
| size | `string` | If specified, creates a ZFS volume of given size (e.g. '10G') |  |

## zfs/remove

```janet
(zfs/remove "old/filesystem")
```

### Mandatory Properties

None

### Optional Properties

None

There is no zfs/remove.