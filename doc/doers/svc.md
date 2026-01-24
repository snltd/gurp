# svc

Manage the state of an existing SMF service.

## svc/ensure

```janet
(svc/ensure "important/service"
            :state "enabled"
            :restarted-by [:/test-role/file/stub])
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| state | `string` | Desired state of service, e.g. 'online' |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| reloaded-by | `array` | Labels of resources whose alteration triggers service reload | <tuple 0x0000024E4BA0> |
| restarted-by | `array` | Labels of resources whose alteration triggers service restart | <tuple 0x0000024E4BD0> |
There is no svc/remove.