# ipfilter

Set or remove ipfilter rules.

## Resource Name

Any convenient name: not used internally (`:string`)

## ipfilter/ensure

```janet
(ipfilter/ensure "rules-from-config"
                 :priority 0
                 :always-reload true
                 :content "block in log all\nblock out all")
```

```janet
(ipfilter/ensure "rules-from-file"
                 :from "test/ipfilter-test"
                 :priority 1)
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:always-reload` | `boolean` | if any ipfilter/ensure resource sets this to true, then the firewall rules will be reloaded every time Gurp runs, regardless of whether the aggregated ipf.conf file has changed |  |
| `:priority` | `number` | rule resources are ordered by priority, lowest number first |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:content` | `string` | Apply these rules. Must have :content xor :from |  |
| `:from` | `string` | Apply rules in the given file. If relative, looks in ../files |  |

## ipfilter/remove

```janet
(ipfilter/remove "removes-all-rules")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- We build a single big set of filter rules from multiple sources, check its validity, and ensure its contents align with those of `/etc/ipf/ipf.conf`. If the file has changed, or if any resource used to build the content has `:always-reloaded true`, the contents of the file become the current firewall configuration.
- The doer automatically enables the ipfilter service.
- We do not (currently) support any additional `ipf` options.
- Per-zone rules are not supported.
- Using :always-reload means Gurp will always show a change to be made
- ipfilter/remove removes ALL filter rules
