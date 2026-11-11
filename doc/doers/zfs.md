# zfs


Create, destroy, and modify properties of ZFS filesystems.

### Resource Name

ZFS filesystem name [`:string`]

## zfs/ensure


### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:checksum` | `struct` | Checksum for `:create-from` target. See `file/checksum`. |  |
| `:create-from` | `string` | If the filesystem does not exist, it will be created from the given snapshot file. Can be a file path or a URL. |  |
| `:properties` | `struct` | ZFS properties (:keyword) paired with desired value (`:string`). | `{}` |
| `:size` | `string` | If specified, creates a ZFS volume of given size (e.g. '10G'). |  |

## zfs/remove


### Mandatory Properties

None

### Optional Properties

None

## Notes


- Gurp does not check parameters are valid, so if you get them wrong the first you'll know about it is when you get an error from `zfs(8)`.

- If you do not set a mountpoint for a filesystem, Gurp will force it to 'none'.

- Gurp cannot change the size of an extant volume.

- `zfs/destroy` is recursive, and will remove all child filesystems and snapshots without asking or telling.

## Examples


```janet
(zfs/ensure "rpool/example"
            :create-from "http://example.com/zfs/example.zfs"
            :properties {:atime false
                         :mtime false
                         :exec false
                         :compression "lz4"}
            (file/checksum
              :algorithm "sha256"
              :literal "--checksum-goes-here--"))
```


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


```janet
(zfs/remove "rpool/old/filesystem")
```
