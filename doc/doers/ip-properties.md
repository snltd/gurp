# ip-properties


Sets global IP properties, via 'ipadm set-prop'.

### Resource Name

Any convenient name: not used internally [`:string`]

## ip-properties/ensure


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

## ip-properties/remove


### Mandatory properties

None

### Optional properties

None

## Notes

- Define `extra_priv_ports` as a comma-separated list.


### Examples

```janet
(ip-properties/ensure "general"
                      :ipv4 {:forwarding true}
                      :ipv6 {:hoplimit 250}
                      :icmp {:max_buf 262000}
                      :tcp {:sack "passive"}
                      :udp {:extra_priv_ports "2050,4040"}
                      :sctp {:max_buf 1048000})
```


