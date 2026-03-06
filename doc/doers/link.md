# link

Create and remove links.

## Resource Name

Qualified path to the link that will be created (`:string`)

## link/ensure

```janet
(link/ensure "/link/is/here"
             :type "hard"
             :source "/link/points/here")
```

```janet
(link/ensure "/symlink/is/here"
             :label "example-symlink"
             :force-link true
             :source "/link/points/here")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:force-link` | `boolean` | If the link target already exists and this flag is true, Gurp will remove it and replace it with a link. If false, a pre-existing target causes an error |  |
| `:source` | `string` | The file to which we will link |  |
| `:type` | `string` | The type of link: symbolic or hard | `"symbolic"` |

### Optional Properties

None

## link/remove

```janet
(link/remove "/dont/want/this/link")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- If the source doesn't exist, you get an error.
- Files and directories are ensured before links, so you can link Gurp-managed resources.
