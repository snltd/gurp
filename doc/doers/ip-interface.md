# ip-interface

Create and destroy IP interfaces, with optional properties. Properties are supplied with 'ip-interface-protocol'.

## ip-interface/ensure

### Resource Name

Interface name (`:string`)

### Mandatory properties

None

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:icmp` | `struct table` | key-value pairs of valid icmp properties. |  |
| `:ip` | `struct table` | key-value pairs of valid ip properties. |  |
| `:ipv4` | `struct table` | key-value pairs of valid ipv4 properties. |  |
| `:ipv6` | `struct table` | key-value pairs of valid ipv6 properties. |  |
| `:sctp` | `struct table` | key-value pairs of valid sctp properties. |  |
| `:tcp` | `struct table` | key-value pairs of valid tcp properties. |  |
| `:udp` | `struct table` | key-value pairs of valid udp properties. |  |

### Examples

```janet
(ip-interface/ensure "example1"
                     :label "example-interface"
                     :ipv6 {:mtu 1500
                            :forwarding false}
                     :ipv4 {:mtu 1500
                            :forwarding true})
```

```janet
(ip-interface/ensure "example0")
```

## ip-interface/remove

### Resource Name

Interface name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(ip-interface/remove "example2")
```

