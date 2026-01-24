# vlan

Manage VLAN objects

## vlan/ensure

```janet
(vlan/ensure "e1000g010"
             :over "e1000g0"
             :vlan-tag 10)
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| over | `string` | Physical link which will serve the VLAN |  |
| vlan-tag | `number` | The VLAN tag ID |  |

### Optional Properties

None

## vlan/remove

```janet
(vlan/remove "old-vlan")
```

### Mandatory Properties

None

### Optional Properties

None

There is no vlan/remove.