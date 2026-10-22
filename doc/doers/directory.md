# directory

Create and remove directories. Parents are created like `mkdir -p`, but with the owner/group/mode of the gurp process. Removal always removes directory contents.

## directory/ensure

### Resource Name

Fully qualified path to directory (`:string`)

### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:group` | `string number` | The group name or GID of the for this directory. | `root` |
| `:mode` | `string` | Permissions, octal. | `0755` |
| `:owner` | `string number` | The username or UID of the user who owns this directory. | `root` |

### Optional properties

None

### Examples

```janet
(directory/ensure "/example/dir_1")
```

```janet
(directory/ensure "/example/dir_3"
                  :label "my-dir"
                  :owner 4
                  :group 12
                  :mode "2750")
```

```janet
(directory/ensure "/example/dir_2"
                  :owner "adm"
                  :group "sys"
                  :mode "0700"
                  :label "all-the-specs")
```

## directory/remove

### Resource Name

Fully qualified path to directory (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(directory/remove "/example")
```

## Notes

- Directories are created/removed in the order of a natural sort.
- Directories are created `mkdir -p` style, but only the mode and owner of the specified directory are managed by Gurp. Any directories 'filled in' to get to the target path will have their ownership and mode dictated by the Gurp process and its umask.
- If you ensure a directory at a path which already exists but is not a directory, Gurp will error.
- Removing a directory removes all its contents, but does not remove any empty ancestors.
