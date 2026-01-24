# svcprop

Manage properties of an existing SMF service.

## svcprop/ensure

```janet
(svcprop/ensure "mariadb"
                :properties {:application/datadir "/data"
                             :application/active true
                             :application/timeout 50})
```

```janet
(svcprop/ensure "mariadb"
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"})
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| properties | `struct` | Properties to create. (:keyword :string|:boolean|:number) |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| property-groups | `struct` | Property groups to create. Key is name, value is type |  |

## svcprop/remove

```janet
(svcprop/remove "mariadb"
                :properties ["application/thing"])
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| properties | `tuple` | Properties to remove |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| property-groups | `struct` | Property groups to remove |  |

There is no svcprop/remove.