# file

Create files from multiple sources, or remove them.

## Resource Name

Fully qualified path to file (`:string`)

## file/ensure

```janet
(file/ensure "/file/from/local_file"
             :group "daemon"
             :mode "0755"
             :from "file-test/does-not-exist")
```

```janet
(file/ensure "/file/from/content"
             :owner "dataperson"
             :mode "0600"
             :content "lots-of-data")
```

```janet
(file/ensure "/file/from/arbitrary/server"
             :owner "gibbus"
             :label "remote-file"
             :mode "0640"
             :with-checksum "0123456789abcdef"
             :from-url "https://example.com/files/config")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:backup-suffix` | `string` | Back up the file with this suff. Use 'TIMESTAMP' for an epoch timestamp |  |
| `:content` | `string` | Literal content of the file. Must have :content xor :from |  |
| `:from-struct` | `struct table tuple` | Generate a config file from the given struct. Requires :to-format |  |
| `:from-url` | `string` | Fetch file from the given URL |  |
| `:from` | `string` | Copy content from this file. If relative, looks in ../files |  |
| `:group` | `string number` | The group name or GID of the for this file | `"root"` |
| `:ignore-pattern` | `string` | When comparing, ignore lines matching this Rust regex |  |
| `:mode` | `string` | Permissions written as a four-digit octal | `"0644"` |
| `:owner` | `string number` | The username or UID of the user who owns this file | `"root"` |
| `:to-format` | `string` | Used with :from-struct to try to turn the struct into this format |  |
| `:with-checksum` | `string` | Blake3 checksum used to validate files fetched by :from-url |  |

## file/remove

```janet
(file/remove "/path/to/file")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- You must supply exactly one of `:content`, `:from`, `:from-url`, or `:from-struct`. If you use `:from-struct` you must also supply `:to-format`.
- The `template-out` and `indoc` macros are useful when specifying :content.
- `:from` takes a fully-qualified or relative path. If you use the latter, Gurp assumes the file is in a ``files/` directory at the same level as the directory holding the file being parsed.
- `:from-struct` and `:to-format` let you turn Janet values into a config file. Fully supported file formats are `json`, `toml`, and `yaml`: these formats can represent any valid struct. You can create INI files (`:to-format "ini"`), but the limits of that format mean your struct must be a struct of structs, each representing a section. An invalid struct will cause an error.
- Gurp can also create key-value pairs (`:to-format "kvp"`). It can do this from a single-level struct, or from an array. In the latter case, entries are alternately keys and values. Using an array lets you create files with duplicate keys, which is sometimes necessary.
