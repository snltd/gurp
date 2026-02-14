# link

Create and remove links.

## Resource Name

Qualified path to the link that will be created (`:string`)

## link/ensure

```janet
(link/ensure "/link/is/here"
             :label "example-link"
             :source "/link/points/here")
```

```janet
(link/ensure "/link/is/here"
             :type "hard"
             :source "/link/points/here")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:source` | `string` | The file to which we will link |  |
| `:type` | `string` | The type of link: symbolic or hard | `"symbolic"` |

### Optional Properties

None

## link/remove

```janet
(ink/remove "/dont/want/this/link")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- If the :source doesn't exist, you get an error.
- Files and directories are ensured before links, so you can link Gurp-managed resources.
- If the link exists and points to the wrong file, it will be removed and re-created, and if it exists but is not a link, that's an error.
