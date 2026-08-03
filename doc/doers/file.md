# file

Create files from multiple sources, or remove them.

## Resource Name

Fully qualified path to file (`:string`)

## file/ensure

```janet
(file/ensure "/example/file/from-content"
             :owner "sys"
             :mode "0600"
             :content "words\n and\nstuff\n")
```

```janet
(file/ensure "/example/file/from-local-file"
             :group "daemon"
             :mode "4755"
             :from "file-dir/example")
```

```janet
(file/ensure "/example/file/from-url"
             :label "remote-file"
             :with-checksum "561a47aa1d1bfc3a95ce45345639f9ce2d9ad332b05cfe5da74ad77f2842ee16"
             :from-url "https://raw.githubusercontent.com/snltd/gurp/refs/heads/main/LICENSE.txt")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:backup-suffix` | `string` | Back up the file with this suffix. Use 'TIMESTAMP' for an epoch timestamp |  |
| `:content` | `string` | Literal content of the file. Must have :content xor :from |  |
| `:from-struct` | `struct table tuple` | Generate a config file from the given struct. Requires :to-format |  |
| `:from-url` | `string` | Fetch file from the given URL |  |
| `:from` | `string` | Copy content from this file. If relative, looks in ../files |  |
| `:group` | `string number` | The group name or GID of the for this file | `"root"` |
| `:ignore-pattern` | `string` | When comparing, ignore lines matching this Rust regex |  |
| `:mode` | `string` | Permissions, octal | `"0644"` |
| `:only-fetch-from-url-once` | `boolean` | If you use :from-url, Gurp must download the file on every run to compare it with the installed copy. When this is set to true, :from-url files are only downloaded if the target file is missing |  |
| `:owner` | `string number` | The username or UID of the user who owns this file | `"root"` |
| `:to-format` | `string` | Used with :from-struct to try to turn the struct into this format |  |
| `:url-is-server` | `boolean` | Used internally to identify Gurp server URLs |  |
| `:url-replacements` | `struct table` | Replace keys with whatever the corresponding value points to |  |
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
- Unless you specify TIMESTAMP, only one backup file is kept. Backup files are always owned by `root:root`, with mode `0400`.
- If you try to ensure a file at a path which exists, but is not a file, Gurp will error
- Gurp can also create key-value pairs (`:to-format "kvp"`). It can do this from a single-level struct, or from an array. In the latter case, entries are alternately keys and values. Using an array lets you create files with duplicate keys, which is sometimes necessary.
