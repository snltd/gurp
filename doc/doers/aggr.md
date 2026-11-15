# aggr


Manage link aggregations

### Resource Name

aggr name [`:string`]

## aggr/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:links` | `tuple` | Underlying links. |  |
| `:policy` | `string` | Port selection policy. | `"L4"` |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:address` | `string` | Unicast hardware address. |  |
| `:lacp-mode` | `string` | LACP mode. Allowed values: "off", "active", "passive". |  |
| `:lacp-timer` | `string` | LACP timer value. Allowed values: "short", "long". |  |

## aggr/remove


### Mandatory Properties

None

### Optional Properties

None

## Notes


- Changing the underlying links forces removal and re-creation of the aggr.

- The hardware address cannot be changed on an existing aggr.

## Examples


```janet
(aggr/ensure "aggr0"
             :lacp-timer "long"
             :policy "L2,L3"
             :links ["vioif1" "vioif2" "vioif3"])
```


```janet
(aggr/remove "aggr1")
```
