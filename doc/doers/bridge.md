# bridge


Create and modify ethernet bridges.

### Resource Name

Any valid bridge name [`:string`]

## bridge/ensure


### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:force-protocol` | `number` | MSTP forced maximum supported protocol. | `3` |
| `:forward-delay` | `number` | STP forward delay time, in seconds. 4 to 30. | `15` |
| `:hello-time` | `number` | STP hello time value, in seconds. | `2` |
| `:max-age` | `number` | Maximum age, in seconds, for STP configuration information. | `20` |
| `:priority` | `number` | Bridge priority. 0 to 61440. | `32768` |
| `:protect` | `string` | Protection method. | `stp` |

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:links` | `tuple` | Existing links which should be attached to the bridge. |  |

## bridge/remove


### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(bridge/ensure "basic")
```
```janet
(bridge/ensure "with_links"
               :links ["stub1" "stub2"]
               :priority 4096
               :max-age 27)
```


```janet
(bridge/remove "unwanted")
```
