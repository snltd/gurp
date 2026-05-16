# publisher

Add and remove pkg(5) publisher origins.

## Resource Name

Publisher name (`:string`)

## publisher/ensure

```janet
(publisher/ensure "example"
                  (publisher/origin "http://pkg.lan.id264.net"
                                    :proxy "http://10.2.0.20/1837")
                  (publisher/mirror "http://mirror.lan.id264.net"))
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:origin` | `array tuple` | List of origins, created with publisher/origin  |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:mirror` | `array tuple` | List of mirrors, created with publisher/mirror |  |
| `:search-first` | `boolean` | Search this publisher first |  |

## publisher/remove

```janet
(publisher/remove "old_publisher")
```

### Mandatory Properties

None

### Optional Properties

None

# publisher/mirror

Define n mirror when managing a publisher.

## Name

The mirror URI (`:string`)

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:name` | `string` | URI of mirror |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:proxy` | `string` | Proxy URI for this mirror |  |


# publisher/origin

Define an origin when managing a publisher.

## Name

The origin URI (`:string`)

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:name` | `string` | URI of origin |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:proxy` | `string` | Proxy URI for this origin |  |

