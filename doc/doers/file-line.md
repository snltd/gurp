# file-line

Ensure lines do or do not exist in the given file.

## Resource Name

Fully qualified path to file (`:string`)

## file-line/ensure

```janet
(file-line/ensure "/path/to/file"
                  :line "i-want-to-see-this")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:apply-to` | `string` | Which matches to act on when replacing: "all", "first", "last" |  |
| `:insert-at` | `number` | If a new line must be added, it will go at this index |  |
| `:line` | `string` | The line which must exist |  |
| `:replace` | `string` | Pattern to replace. Rust regex |  |
| `:with` | `string` | Counterpart to :replace |  |

## file-line/remove

```janet
(file-line/remove "/path/to/file"
                  :pattern "i-do-not-want-to-see-this-anywhere")
```

```janet
(file-line/remove "/path/to/file"
                  :match "regex"
                  :pattern "rust-regex")
```

```janet
(file-line/remove "/path/to/file"
                  :pattern "string-prefix"
                  :match "starts-with"
                  :apply-to "last")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:apply-to` | `string` | Which matches to act on: "all", "first", "last" | `"all"` |
| `:match` | `string` | How to match the line: "exact", "starts-with", "ends-with", "contains", "regex" | `"exact"` |
| `:pattern` | `string` | The line or pattern which must be removed |  |

### Optional Properties

None

## Notes

- The file is not managed here. Use a file resource.
- The doer reads the whole file into memory, so be mindful of file size.
- Appended lines have a newline at the beginning and end.
- Removing a line puts a newline on the end of the file if there wasn't one already.
- Files are not backed up.
