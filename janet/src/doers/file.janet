(use ./lib/definers)
(import ../collector)
(import ./file/checksum :prefix "" :export true)

(defdoer :file
  "Create files from multiple sources, or remove them."
  :name-is "Fully qualified path to file"

  (ensure
    :exactly-one-of [:content :from :from-url :from-struct]
    :expand-resources {:checksum true}

    (property "mode" [:string] :mandatory "Permissions, octal"
              :default "0644")
    (property "owner" [:string :number] :mandatory
              "The username or UID of the user who owns this file"
              :default "root")
    (property "group" [:string :number] :mandatory
              "The group name or GID of the for this file"
              :default "root")
    (property "backup-suffix" [:string] :optional
              "Back up the file with this suffix. Use `TIMESTAMP` for an epoch
               timestamp")
    (property "content" [:string] :optional "Literal content of the file")
    (property "from" [:string] :optional
              "Copy content from this file. If relative, looks in `../files`")
    (property "from-struct" [:struct :tuple] :optional
              "Generate a config file from the given struct. Requires `:to-format`")
    (property "to-format" [:string] :optional
              "Used with `:from-struct` to try to turn the struct into this format"
              :values ["json" "toml" "yaml" "init" "kvp"])
    (property "from-url" [:string] :optional "Fetch file from the given URL")
    (property "ignore-pattern" [:string] :optional
              "When comparing, ignore lines matching this Rust regex")
    (property "url-replacements" [:struct] :optional
              "Replace keys with whatever the corresponding value points to. Not
               supported with `:from-url`")
    (property "checksum" [:struct] :optional "See file/checksum")
    (property "url-is-server" [:boolean] :optional
              "Used internally to identify Gurp server URLs")
    (property "only-fetch-from-url-once" [:boolean] :optional
              "If you use `:from-url`, Gurp must download the file on every run
               to compare it with the installed copy. When this is set to
               `true`, `:from-url` files are only downloaded if the target file
               is missing"))

  (notes
    "`:from` takes a fully-qualified or relative path. If you use the latter,
     Gurp assumes the file is in a ``files/` directory at the same level as the
     directory holding the file being parsed."
    "`:from-struct` and `:to-format` let you turn Janet values into a config
     file. Fully supported file formats are `json`, `toml`, and `yaml`: these
     formats can represent any valid struct. You can create INI files
     (`:to-format \"ini\"`), but the limits of that format mean your struct
     must be a struct of structs, each representing a section. An invalid struct
     will cause an error."
    "Unless you specify `TIMESTAMP`, only one backup file is kept. Backup files
     are always owned by `root:root`, with mode `0400`."
    "If you try to ensure a file at a path which exists, but is not a file, Gurp
     will error"
    "The `template-out` and `indoc` macros are useful when specifying :content."
    "Gurp can also create key-value pairs (`:to-format \"kvp\"`). It can do this
     from a single-level struct, or from an array. In the latter case, entries
     are alternately keys and values. Using an array lets you create files with
     duplicate keys."))

(make-fn :ensure :file)
(make-fn :remove :file)
