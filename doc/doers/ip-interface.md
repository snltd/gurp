# ip-interface

Create and destroy IP interfaces, with optional properties.                 Properties are supplied with 'ip-interface-protocol'.

## ip-interface/ensure

```janet
(ip-interface/ensure "test-vnic1"
                     :label "merp"
                     (ip-interface/protocol "ipv6"
                                            :mtu 1500
                                            :forwarding false)
                     (ip-interface/protocol "ipv4"
                                            :mtu 1500
                                            :forwarding true))
```

```janet
(ip-interface/ensure "test-vnic0")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| protocols | `struct table` | See 'ip-interface-protocol' |  |

## ip-interface/remove

```janet
(ip-interface/remove "test-vnic3")
```

### Mandatory Properties

None

### Optional Properties

None

There is no ip-interface/remove.