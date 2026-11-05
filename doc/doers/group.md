# group


Create and destroy Unix groups.

### Resource Name

Group name [`:string`]

## group/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gid` | `number` | The group ID. |  |

### Optional Properties

None

## group/remove


### Mandatory Properties

None

### Optional Properties

None

## Examples


```janet
(group/ensure "new-group"
              :gid 264)
```


```janet
(group/remove "old-group")
```
