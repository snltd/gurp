# ip-properties

Sets global IP properties, via 'ipadm set-prop'.

## Resource Name

Any convenient name: not used internally (`:string`)

## ip-properties/ensure

```janet
(ip-properties/ensure "general"
                      :ipv6 {:hoplimit 123
                             :hostmodel "weak"}
                      :ipv4 {:hostmodel "weak"}
                      :icmp {:max_buf 1234567})
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

## ip-properties/remove

There is no ip-properties/remove.