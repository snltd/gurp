# symlink

Create and remove symbolic links.

## Resource Name

Qualified path to the link that will be created (`:string`)

## symlink/ensure

```janet
(symlink/ensure "/link/is/here"
                :label "example-link"
                :source "/link/points/here")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:source` | `string` | The file the symlink points to |  |

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

## Notes

- If the :source doesn't exist, you get an error.
- Files are ensured before links, so you can make a file and link to it.
- If the link exists and points to the wrong file, it will be removed and re-created, and if it exists but is not a link, that's an error.
