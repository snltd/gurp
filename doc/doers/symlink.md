# symlink

Create and remove symbolic links.

## symlink/ensure

```janet
(symlink/ensure "/link/is/here"
                :label "test-link"
                :source "/link/points/here")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| source | `string` | The file the symlink points to |  |

### Optional Properties

None

## symlink/remove

```janet
(symlink/remove "/dont/want/this/link")
```

### Mandatory Properties

None

### Optional Properties

None

There is no symlink/remove.