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

## Notes


- `ipmp/ensure` assumes that the interfaces it is given already exist. It is your responsibility to create them, with the `ip-interface` doer

## Examples


```janet
(def interfaces ["mh0" "mh1" "mh2"])

(each iface interfaces
  (ip-interface/ensure iface))

(ipmp/ensure "ipmp0" :interfaces interfaces)
```


```janet
(ipmp/remove "ipmp1")
```
