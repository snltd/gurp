# file-line

Ensure lines do or do not exist in the given file.

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
| apply-to | `string` | Which matches to act on when replacing: "all", "first", "last" |  |
| insert-at | `number` | If a new line must be added, it will go at this index |  |
| line | `string` | The line which must exist |  |
| replace | `string` | Pattern to replace. Rust regex |  |
| with | `string` | Counterpart to :replace |  |

## file-line/remove

```janet
(file-line/remove "/tmp/.tmpjpqQir/test-file"
                  :pattern "line_2"
                  :match "exact"
                  :apply-to "all")
```

```janet
(file-line/remove "/path/to/file"
                  :pattern "this-is-an-awful-line")
```

```janet
(file-line/remove "/path/to/file"
                  :match "exact"
                  :apply-to "last"
                  :pattern "this-is-an-awful-line")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| pattern | `string` | The line or pattern which must be removed |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| apply-to | `string` | Which matches to act on: "all", "first", "last" | all |
| match | `string` | How to match the line: "exact", "starts_with", "ends_with", "contains", "matches" | exact |

There is no file-line/remove.