# ipnat

Set or remove NAT rules.

## Resouce Name

Any convenient name: not used internally (`:string`)

## ipnat/ensure

```janet
(ipnat/ensure "rules-in-config"
              :priority 1
              :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\n
rdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin")
```

```janet
(ipnat/ensure "rules-in-file"
              :from "test/ipnat-test"
              :priority 2)
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:priority` | `number` | NAT rule resources are ordered by priority, lowest number first |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:content` | `string` | Apply these rules. Must have :content xor :from |  |
| `:from` | `string` | Apply rules in the given file. If relative, looks in ../files |  |

## ipnat/remove

```janet
(ipnat/remove "removes-all-rules")
```

### Mandatory Properties

None

### Optional Properties

None

