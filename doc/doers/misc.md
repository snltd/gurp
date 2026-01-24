# misc

A collection of things too small to deserve their own doer.

## misc/ensure

```janet
(misc/ensure :nfs-domain "lan.id264.net")
```

```janet
(misc/ensure :scheduler "FSS")
```

```janet
(misc/ensure :enable-smb "rob")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| enable-smb | `string` | Enable SMB sharing for this username |  |
| nfs-domain | `string` | NFS domain name |  |
| scheduler | `string` | The scheduler class to set via dispamdin |  |
There is no misc/remove.