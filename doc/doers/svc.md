# svc

Manage the state of an existing SMF service.

## Resouce Name

Service FMRI (`:string`)

## svc/ensure

```janet
(svc/ensure "important/service"
            :state "enabled"
            :restarted-by [:/test-role/file/stub])
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:state` | `string` | Desired state of service, e.g. 'online' |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:reloaded-by` | `array` | Labels of resources whose alteration triggers service reload | `()` |
| `:restarted-by` | `array` | Labels of resources whose alteration triggers service restart | `()` |

## svc/remove

There is no svc/remove.