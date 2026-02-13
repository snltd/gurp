# vnic

Manage VNIC objects

## Resource Name

VNIC name (`:string`)

## vnic/ensure

```janet
(vnic/ensure "vnic0"
             :over "e1000g")
```

```janet
(vnic/ensure "vnic1"
  :over "e1000g"
  :vlan-tag 10
  :with-interface true)
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:over` | `string` | Physical link which will serve the VNIC |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:vlan-tag` | `number` | Enable VLAN tagging with the given tag |  |
| `:with-interface` | `boolean` | Whether to create an IP interface on the new VNIC |  |

## vnic/remove

```janet
(vnic/remove "vnic2")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- VNICs get a random MAC address.
- If a VNIC exists but has a different VLAN tag or underlying physical NIC to the specification, Gurp will try to recreate it.
