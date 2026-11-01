# vlan


Manage VLAN objects

### Resource Name

VLAN name [`:string`]

## vlan/ensure


### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:over` | `string` | Physical link which will serve the VLAN. |  |
| `:vlan-tag` | `number` | The VLAN tag ID. |  |

### Optional properties

None

## vlan/remove


### Mandatory properties

None

### Optional properties

None

## Notes

- You must specify the VLAN link name. Gurp does not support automatic naming, as it goes against its policy of assuming nothing.


### Examples

```janet
(vlan/ensure "e1000g10000"
             :over "e1000g0"
             :vlan-tag 10)
```


```janet
(vlan/remove "e1000g10000")
```
