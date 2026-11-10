# directory


Create and remove directories. Parents are created like `mkdir -p`, but with the owner/group/mode of the gurp process. Removal always removes directory contents.

### Resource Name

Fully qualified path to directory [`:string`]

## directory/ensure


### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:group` | `string number` | The group name or GID the for this directory. | `"root"` |
| `:mode` | `string` | Permissions, octal. | `"0755"` |
| `:owner` | `string number` | The username or UID of the user who owns this directory. | `"root"` |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:checksum` | `struct` | Checksum of archive when using :create-from or create-with. See `file/checksum`. |  |
| `:create-from` | `string` | Create the directory by unpacking the given archive. |  |
| `:create-with` | `string` | Create the directoey and unpack the given archive inside it. |  |

## directory/remove


### Mandatory Properties

None

### Optional Properties

None

## Notes


- Directories are created/removed in the order of a natural sort.

- Directories are created `mkdir -p` style, but only the mode and owner of the specified directory are managed by Gurp. Any directories 'filled in' to get to the target path will have their ownership and mode dictated by the Gurp process and its umask.

- If you ensure a directory at a path which already exists but is not a directory, Gurp will error.

- Removing a directory removes all its contents, but does not remove any empty ancestors.

- `:create-from` and `:create-with` only apply when the directory is created. They will not modify an existing directory, and Gurp will not automatically manage any files created by unpacking the archive.

- `:create-from` and `:create-with` require GNU `tar`.

## Examples


```janet
(directory/ensure "/example/dir_1")
```


```janet
(directory/ensure "/example/dir_4"
                  :create-from "http://example.com/files/archive.tar"
                  (file/checksum
                    :algorithm "sha512"
                    :suffix "sha512"))
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


```janet
(directory/remove "/example")
```
