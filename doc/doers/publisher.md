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
| `:uri` | `string` | Add a pkg publiser with this URI |  |

### Optional Properties

None

## publisher/remove

```janet
(publisher/remove "old_publisher")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- Only handles origins.
