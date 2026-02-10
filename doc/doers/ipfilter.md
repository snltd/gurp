# ipfilter

Set or remove ipfilter rules.

## Resource Name

Any convenient name: not used internally (`:string`)

## ipfilter/ensure

```janet
(ipfilter/ensure "rules-from-file"
                 :from "test/ipfilter-test"
                 :priority 1)
```

```janet
(ipfilter/ensure "rules-in-config"
                 :priority 0
                 :always-reload true
                 :content "block in log all\nblock out all")
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

