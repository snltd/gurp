# svcprop

Set and remove properties and property groups of an existing SMF service.

## Resource Name

Any valid FMRI of the service whose properties you wish to set (`:string`)

## svcprop/ensure

```janet
(svcprop/ensure "example/svc_1"
                :on-change "restart"
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"
                             :application/active true
                             :application/timeout 50})
```

```janet
(svcprop/ensure "example/svc_1"
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"})
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:properties` | `struct` | Properties to create. (:keyword :string|:boolean|:number) |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:on-change` | `string` | Take this action when a value is changed. One of restart, refresh |  |
| `:property-groups` | `struct` | Property groups to create. Key is name, value is type |  |

## svcprop/remove

```janet
(svcprop/remove "example/svc_3"
                :properties ["application/thing"])
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:properties` | `tuple array` | Properties to remove |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:property-groups` | `tuple array` | Property groups to remove |  |

## Notes

- If you want to change a property value on a service instance, you may also have to define the property group to which it belongs, as it may not be inherited from the base service.
- When a service restarts on-change, it also refreshes.
- If not specified, Gurp will infer the types of property values.
- You can't change the type of an existing property group.
