# aggr


Manage link aggregations

### Resource Name

aggr name [`:string`]

## aggr/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:links` | `tuple` | Underlying links. |  |
| `:policy` | `string` | Port selection policy. Allowed values: "L2", "L3", "L4". | `"L4"` |

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

## Examples
