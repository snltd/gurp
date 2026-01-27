# group

Create and destroy Unix groups.

## Resource Name

Group name (`:string`)

## group/ensure

```janet
(group/ensure "new-group"
              :gid 264)
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gid` | `number` | The group ID |  |

### Optional Properties

None

## group/remove

```janet
(group/remove "old-group")
```

### Mandatory Properties

None

### Optional Properties

None

