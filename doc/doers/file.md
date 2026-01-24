# file

Create files from multiple sources, or remove them.

## Resouce Name

Fully qualified path to file (`:string`)

## file/ensure

```janet
(file/ensure "/path/to/file"
             :group "daemon"
             :mode "0755"
             :from "file-test/does-not-exist")
```

```janet
(file/ensure "/file/path"
             :owner "dataperson"
             :mode "0600"
             :content "lots-of-data")
```

```janet
(file/ensure "/file/from/remote/path"
             :owner "gibbus"
             :mode "0640"
             :with-checksum "0123456789abcdef"
             :from-url "https://example.com/files/config")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:backup-suffix` | `string` | Back up the file with this suff. Use 'TIMESTAMP' for                          an epoch timestamp |  |
| `:content` | `string` | Literal content of the file. Must have :content xor :from |  |
| `:from-struct` | `struct table tuple` | Generate a config file from the given struct. Requires                       :to-format |  |
| `:from-url` | `string` | Fetch file from the given URL |  |
| `:from` | `string` | Copy content from this file. If relative, looks in ../files |  |
| `:group` | `string number` | The group name or GID of the for this file | `"root"` |
| `:ignore-pattern` | `string` | When comparing, ignore lines matching this Rust regex |  |
| `:mode` | `string` | Permissions written as a four-digit octal | `"0644"` |
| `:owner` | `string number` | The username or UID of the user who owns this file | `"root"` |
| `:to-format` | `string` | Used with :from-struct to try to turn the struct into this                     format |  |
| `:with-checksum` | `string` | Blake3 checksum used to validate files fetched by                         :from-url |  |

## file/remove

```janet
(file/remove "/path/to/file")
```

### Mandatory Properties

None

### Optional Properties

None

