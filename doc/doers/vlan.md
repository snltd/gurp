# vlan

Manage VLAN objects

## vlan/ensure

### Resource Name

VLAN name (`:string`)

### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:over` | `string` | Physical link which will serve the VLAN. |  |
| `:vlan-tag` | `number` | The VLAN tag ID. |  |

### Optional properties

None

### Examples

```janet
(vlan/ensure "e1000g10000"
             :over "e1000g0"
             :vlan-tag 10)
```

## vlan/remove

### Resource Name

VLAN name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(vlan/remove "e1000g10000")
```

## Notes

- You must specify the VLAN link name. Gurp does not support automatic naming, as it goes against its policy of assuming nothing.
