# bridge

Create and modify ethernet bridges.

## Resource Name

Any valid bridge name (`:string`)

## bridge/ensure

```janet
(bridge/ensure "basic")
```

```janet
(bridge/ensure "with_links"
               :links ["stub1" "stub2"]
               :priority 4096
               :max-age 27)
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:force-protocol` | `number` | MSTP forced maximum supported protocol |  |
| `:forward-delay` | `number` | STP forward delay time, in seconds. 4 to 30 |  |
| `:hello-time` | `number` | STP hello time value, in seconds |  |
| `:max-age` | `number` | Maximum age, in seconds, for STP configuration information |  |
| `:priority` | `number` | Bridge priority. 0 to 61440 |  |
| `:protect` | `string` | Protection method |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:links` | `tuple array` | Existing links which should be attached to the bridge |  |

## bridge/remove

```janet
(bridge/remove "unwanted")
```

### Mandatory Properties

None

### Optional Properties

None

