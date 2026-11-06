# ip-address


Manages IP addresses via ipadm.

### Resource Name

Address name, e.g. vnic0/v4 [`:string`]

## ip-address/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:type` | `string` | Type of address. Allowed values: "static", "dhcp". |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:address` | `string` | Local IP address with /netmask, if using static address. |  |
| `:properties` | `struct` | Struct of any valid ipadm addrprops. |  |

## ip-address/remove


### Mandatory Properties

None

### Optional Properties

None

## Examples


```janet
(ip-address/ensure "example1/v4"
                   :type "dhcp")
```


```janet
(ip-address/ensure "example0/v4"
                   :type "static"
                   :address "192.168.1.13/24"
                   :properties {:prefixlen 24
                                :transmit true
                                :private false})
```


```janet
(ip-address/remove "example2/v4")
```
