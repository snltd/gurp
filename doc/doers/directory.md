# directory

Create and remove directories. Parents are created like mkdir -p, but with the owner/group/mode of the gurp process. Removal always removes directory contents.

## Resource Name

Fully qualified path to directory (`:string`)

## directory/ensure

```janet
(directory/ensure "/path/to/dir_1")
```

```janet
(directory/ensure "/path/to/dir_2"
                  :label "my-dir"
                  :mode "0700")
```

```janet
(directory/ensure "/path/to/dir_3"
                  :owner "myself"
                  :group "sysadmin"
                  :mode "0700"
                  :label "all-the-specs")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:group` | `string number` | The group name or GID of the for this directory | `"root"` |
| `:mode` | `string` | Permissions, written as a four-digit octal | `"0755"` |
| `:owner` | `string number` | The username or UID of the user who owns this directory | `"root"` |

### Optional Properties

None

## directory/remove

```janet
(directory/remove "/path/to/dir")
```

### Mandatory Properties

None

### Optional Properties

None

