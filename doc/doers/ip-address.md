# ip-address

Manages IP addresses via ipadm.

## Resouce Name

Address name, e.g. vnic0/v4 (`:string`)

## ip-address/ensure

```janet
(ip-address/ensure "example0/v4"
                   :type "static"
                   :address "192.168.1.13/24"
                   :properties {:prefixlen 24
                                :transmit true
                                :private false})
```

```janet
(ip-address/ensure "example1/v4"
                   :type "dhcp")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:type` | `string` | Type of connection: 'static', 'dhcp' |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:address` | `string` | Local IP address with /netmask, if using static address |  |
| `:properties` | `struct` | Struct of any valid ipadm addrprops |  |

## ip-address/remove

```janet
(ip-address/remove "example3/v4")
```

### Mandatory Properties

None

### Optional Properties

None

