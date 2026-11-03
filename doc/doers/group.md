# group


Create and destroy Unix groups.

### Resource Name

Group name [`:string`]

## group/ensure


### mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gid` | `number` | The group ID. |  |

### Optional properties

None

## group/remove


### mandatory properties

None

### Optional properties

None

## Examples


```janet
(group/ensure "new-group"
              :gid 264)
```


```janet
(group/remove "old-group")
```
