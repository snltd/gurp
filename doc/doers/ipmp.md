# ipmp


Manage IPMP interfaces

### Resource Name

IPMP interface name [`:string`]

## ipmp/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:interfaces` | `tuple` | Interfaces which should form IPMP group. |  |

### Optional Properties

None

## ipmp/remove


### Mandatory Properties

None

### Optional Properties

None

## Examples


```janet
(ipmp/ensure "ipmp0"
             :interfaces ["bge0" "e1000g0"])
```


```janet
(ipmp/remove "ipmp1")
```
