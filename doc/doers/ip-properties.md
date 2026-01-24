# ip-properties

Sets global IP properties, via 'ipadm set-prop'.

## ip-properties/ensure

```janet
(ip-properties/ensure "general"
                      :properties {:ipv6 {:hoplimit 123
                                          :hostmodel "weak"}
                                   :ipv4 {:hostmodel "weak"}
                                   :icmp {:max_buf 1234567}})
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| properties | `struct` | A struct whose keys are protocols (e.g. 'ipv4', 'ipv6'),                      and whose values are structs pairing properties (e.g.                      :hoplimit, :max_buf) with values |  |
There is no ip-properties/remove.