# vnic

Manage VNIC objects

## vnic/ensure

```janet
(vnic/ensure "test-vnic0"
             :over "e1000g")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| over | `string` | Physical link which will serve the VNIC |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| vlan-tag | `number` | Enable VLAN tagging with the given tag |  |
| with-interface | `boolean` | Whether to create an IP interface on the new VNIC | false |

## vnic/remove

```janet
(vnic/remove "test-vnic1")
```

### Mandatory Properties

None

### Optional Properties

None

There is no vnic/remove.