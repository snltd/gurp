# svc


Manage the state of an existing SMF service.

### Resource Name

Service FMRI [`:string`]

## svc/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:state` | `string` | Desired state of service, e.g. `online`. |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:reloaded-by` | `tuple` | Labels of resources whose alteration triggers service reload. |  |
| `:restarted-by` | `tuple` | Labels of resources whose alteration triggers service restart. |  |

## svc/remove


### Mandatory Properties

None

### Optional Properties

None

## Notes


- Because Gurp ends up shelling out to `svcs` and `svcadm`, the service name can be any valid FMRI

## Examples


```janet
(svc/ensure "snltd/example"
            :state "disabled")
```


```janet
(svc/ensure "important/service"
            :state "enabled"
            :restarted-by [:/test-role/file/stub])
```
