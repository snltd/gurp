# group

Create and destroy Unix groups.

## group/ensure

### Resource Name

Group name (`:string`)

### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gid` | `number` | The group ID. |  |

### Optional properties

None

### Examples

```janet
(group/ensure "new-group"
              :gid 264)
```

## group/remove

### Resource Name

Group name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(group/remove "old-group")
```

