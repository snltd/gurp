# route

Manage routes. Note that default routes for zones should be handled by the zone's :defrouter property.

## Resource Name

The route destination, e.g. '10.10.0.0/16'. For a default route            specify 'default'. (`:string`)

## route/ensure

```janet
(route/ensure "203.0.113.0/24"
              :gateway "127.0.0.1"
              :type "blackhole")
```

```janet
(route/ensure "default"
              :gateway "192.168.1.1")
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
| `:flags` | `struct` | Key-value pairs for flags. If the flag does not take a value, use true |  |
| `:force-gateway` | `boolean` | If true, put '-gateway' before the gateway to remove ambiguity |  |
| `:gateway` | `string` | Gateway for given route. |  |
| `:interface` | `string` | Interface for given route. Conflicts with :gateway |  |
| `:type` | `string` | Type of route: e.g. 'blackhole', 'reject' |  |

## route/remove

```janet
(route/remove "203.0.113.0/24"
              :gateway "127.0.0.1"
              :type "blackhole")
```

```janet
(route/remove "10.0.5.0/24"
              :gateway "10.0.5.150")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gateway` | `string` | Gateway for given route. |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:type` | `string` | Type of route: e.g. 'blackhole', 'reject' |  |

## Notes

- The `route` command is messy legacy, and it takes all manner of commands. This is a best-guess attempt to provide something useful.
- We only add persistent routes.
- We only support IPv4.
- If you created a route of a specific type (e.g. blackhole) be sure to also specify the type if you remove it. Otherwise the OS route command can get in a tangle.
- Flags only get set when a route is created. We can't change them on an existing route.
