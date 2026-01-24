# route

Manage routes. Note that default routes for zones should be                 handled by the zone's :defrouter property.

## Resouce Name

The route destination, e.g. 10.10.0.0/16 (`:string`)

## route/ensure

```janet
(route/ensure "192.168.1.1" :gateway "default")
```

```janet
(route/ensure "203.0.113.0/24"
              :gateway "127.0.0.1"
              :type "blackhole")
```

```janet
(route/ensure "10.0.5.0/24"
              :gateway "10.0.5.150"
              :flags {:mtu 1500})
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:flags` | `struct` | Key-value pairs for flags. If the flag does not take a value,                  use true |  |
| `:force-gateway` | `boolean` | If true, put '-gateway' before the gateway to remove                         ambiguity |  |
| `:gateway` | `string` | Gateway for given route. For a default route specify                   'default' |  |
| `:interface` | `string` | Interface for given route. Conflicts with :gateway |  |
| `:type` | `string` | Type of route: e.g. 'blackhole', 'reject' |  |

## route/remove

```janet
(route/remove "10.0.5.0/24" :gateway "10.0.5.150")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gateway` | `string` | Gateway for given route. For a default route specify                   'default' |  |

### Optional Properties

None

