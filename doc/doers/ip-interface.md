# ip-interface


Create and destroy IP interfaces, with optional properties.

### Resource Name

Interface name [`:string`]

## ip-interface/ensure


### Mandatory properties

None

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:icmp` | `struct` | key-value pairs of valid icmp properties. |  |
| `:ip` | `struct` | key-value pairs of valid ip properties. |  |
| `:ipv4` | `struct` | key-value pairs of valid ipv4 properties. |  |
| `:ipv6` | `struct` | key-value pairs of valid ipv6 properties. |  |
| `:sctp` | `struct` | key-value pairs of valid sctp properties. |  |
| `:tcp` | `struct` | key-value pairs of valid tcp properties. |  |
| `:udp` | `struct` | key-value pairs of valid udp properties. |  |

## ip-interface/remove


### Mandatory properties

None

### Optional properties

None

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


```janet
(ip-interface/remove "example2")
```
