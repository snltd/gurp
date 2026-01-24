# bridge

Create and modify ethernet bridges.

## bridge/ensure

```janet
(bridge/ensure "test_a")
```

```janet
(bridge/ensure "test_b"
               :links ["stub0" "vnic0" "e1000g0"]
               :priority 4096
               :max-age 30)
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| force-protocol | `number` | MSTP forced maximum supported protocol | 3 |
| forward-delay | `number` | STP forward delay time, in seconds. 4 to 30 | 15 |
| hello-time | `number` | STP hello time value, in seconds | 2 |
| links | `tuple array` | Existing links which should be attached to the bridge |  |
| max-age | `number` | Maximum age, in seconds, for STP configuration information. | 20 |
| priority | `number` | Bridge priority. 0 to 61440 | 32768 |
| protect | `string` | Protection method: defaults to stp | stp |

## bridge/remove

```janet
(bridge/remove "test_c")
```

### Mandatory Properties

None

### Optional Properties

None

There is no bridge/remove.