# vnic


Manage VNIC objects

### Resource Name

VNIC name [`:string`]

## vnic/ensure


### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:over` | `string` | Physical link which will serve the VNIC. |  |

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:vlan-tag` | `number` | Enable VLAN tagging with the given tag. |  |
| `:with-interface` | `boolean` | Whether to create an IP interface on the new VNIC. |  |

## vnic/remove


### Mandatory properties

None

### Optional properties

None

## Notes

- VNICs get a random MAC address.
- If a VNIC exists but has a different VLAN tag or underlying physical NIC to the specification, Gurp will try to recreate it.


### Examples

```janet
(vnic/ensure "vnic0"
             :over "e1000g0")
```
```janet
(vnic/ensure "vnic1"
             :over "e1000g0"
             :vlan-tag 10
             :with-interface true)
```


```janet
(vnic/remove "vnic2")
```
