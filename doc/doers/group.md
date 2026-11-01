# group


Create and destroy Unix groups.

### Resource Name

Group name [`:string`]

## group/ensure


### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gid` | `number` | The group ID. |  |

### Optional properties

None

## group/remove


### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(group/ensure "new-group"
              :gid 264)
```


```janet
(group/remove "old-group")
```
