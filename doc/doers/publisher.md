# publisher

Add and remove pkg(5) publisher origins.

## Resource Name

Publisher name (`:string`)

## publisher/ensure

```janet
(publisher/ensure "new_publisher"
                  :uri "http://pkg.lan.id264.net")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:type` | `string` | Publisher type: one of "origin", "mirror" | `"origin"` |
| `:uri` | `string` | Add a pkg publisher with this URI |  |

### Optional Properties

None

## publisher/remove

```janet
(publisher/remove "old_publisher")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:mirror` | `string` | Remove the mirror with the given URI |  |

