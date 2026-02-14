# misc

A collection of things too small to deserve their own doer.

## Resource Name

This doer does not accept a name

## misc/ensure

```janet
(misc/ensure
  :nfs-domain "lan.id264.net")
```

```janet
(misc/ensure
  :enable-smb "rob")
```

```janet
(misc/ensure
  :scheduler "FSS")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:enable-smb` | `string` | Enable SMB sharing for this username |  |
| `:nfs-domain` | `string` | NFS domain name |  |
| `:scheduler` | `string` | The scheduler class to set via dispamdin |  |

## misc/remove

There is no misc/remove.## Notes

- The misc doer is a placeholder for what Gurp considers "OS primitives" but which are not big or complex enough to warrant their own doer
