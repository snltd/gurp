# ipnat

Set or remove NAT rules.

## Resource Name

Any convenient name: not used internally (`:string`)

## ipnat/ensure

```janet
(ipnat/ensure "rules-in-file"
              :from "test/ipnat-test"
              :priority 2)
```

```janet
(ipnat/ensure "rules-in-config"
              :priority 1
              :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\n
rdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin")
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

## Notes

- Gurp assembles bundles of NAT rules, given as literal strings (`:content`) or in files (`:from`). Ordering comes from the `:priority` value, lowest first. When all rules have been assembled, the list is verified and compared against the currently loaded NAT rules. If different, the new rules are applied and written to `/etc/ipf/ipnat.conf`.
- Every run asserts the live and persistent state of the NAT table.
- No ipnat flags (-R, -r etc) are supported.
- It's too tricky to support local-zone-from-global-zone NAT rules, so we don't.
- The doer automatically enables the ipfilter service.
- ipnat/remove removes ALL NAT rules
