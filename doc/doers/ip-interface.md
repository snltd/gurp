# ip-interface

Create and destroy IP interfaces, with optional properties.                 Properties are supplied with 'ip-interface-protocol'.

## Resource Name

Interface name (`:string`)

## ip-interface/ensure

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

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:icmp` | `struct table` | key-value pairs of valid icmp properties |  |
| `:ip` | `struct table` | key-value pairs of valid ip properties |  |
| `:ipv4` | `struct table` | key-value pairs of valid ipv4 properties |  |
| `:ipv6` | `struct table` | key-value pairs of valid ipv6 properties |  |
| `:sctp` | `struct table` | key-value pairs of valid sctp properties |  |
| `:tcp` | `struct table` | key-value pairs of valid tcp properties |  |
| `:udp` | `struct table` | key-value pairs of valid udp properties |  |

## ip-interface/remove

```janet
(ip-interface/remove "example3")
```

### Mandatory Properties

None

### Optional Properties

None

