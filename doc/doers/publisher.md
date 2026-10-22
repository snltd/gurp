# publisher

Add and remove pkg(5) publisher origins.

## publisher/ensure

### Resource Name

Publisher name (`:string`)

### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:origin` | `array tuple` | List of origins, created with publisher/origin . |  |

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:mirror` | `array tuple` | List of mirrors, created with publisher/mirror. |  |
| `:search-first` | `boolean` | Search this publisher first. |  |

### Examples

```janet
(publisher/ensure "example"
                  (publisher/origin "http://pkg.lan.id264.net"
                                    :proxy "http://10.2.0.20/1837")
                  (publisher/mirror "http://mirror.lan.id264.net"))
```

## publisher/remove

### Resource Name

Publisher name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(publisher/remove "old_publisher")
```

