(use ./lib)
(use ../dsl)
(import ../collector)

(def client-api-version "v1")
(def doer :file)
(def description "Create files from multiple sources, or remove them.")
(def name-is "Fully qualified path to file")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:backup-suffix
   {:types [:string]
    :help "Back up the file with this suffix. Use 'TIMESTAMP' for an epoch
           timestamp"}
   :from
   {:types [:string]
    :help "Copy content from this file. If relative, looks in ../files"}
   :from-struct
   {:types [:struct :table :tuple]
    :help "Generate a config file from the given struct. Requires :to-format"}
   :from-url
   {:types [:string]
    :help "Fetch file from the given URL"}
   :group
   {:types [:string :number]
    :help "The group name or GID of the for this file"}
   :ignore-pattern
   {:types [:string]
    :help "When comparing, ignore lines matching this Rust regex"}
   :mode
   {:types [:string]
    :help "Permissions written as a four-digit octal"}
   :owner
   {:types [:string :number]
    :help "The username or UID of the user who owns this file"}
   :to-format
   {:types [:string]
    :help "Used with :from-struct to try to turn the struct into this format"}
   :with-checksum
   {:types [:string]
    :help "Blake3 checksum used to validate files fetched by :from-url"}
   :only-fetch-from-url-once
   {:types [:boolean]
    :help "If you use :from-url, Gurp must download the file on every run to
           compare it with the installed copy. When this is set to true,
           :from-url files are only downloaded if the target file is missing"}
   :url-is-server
   {:types [:boolean]
    :help "Used internally to identify Gurp server URLs"}
   :content
   {:types [:string]
    :help "Literal content of the file. Must have :content xor :from"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:owner "root"
   :mode "0644"
   :group "root"})
(def defaults-remove {})

(defn- server-url
  [server-name from-path]
  (string "http://" server-name "/" client-api-version "/file/" from-path))

(defn ensure
  "Given a file path and spec, put an ensure struct in the collector. If Gurp is
   running as a remote client, changes local file references into HTTP ones."
  [name & spec]
  (def spec-table (struct/to-table (make-spec-struct ;spec)))

  (if-let [from-path (spec-table :from)]
    (if-let [server-name (dyn :server-name)]
      (do
        (set (spec-table :from) nil)
        (set (spec-table :from-url) (server-url server-name from-path))
        (set (spec-table :url-is-server) true))
      (let [url-or-qualified-path
            (if (string/find "://" from-path)
              from-path
              (qualify-from-path from-path))]
        (set (spec-table :from) url-or-qualified-path))))

  (def all-specs (spec-with-defaults defaults-ensure spec-table))
  (def safe-specs
    (pinpoint-error
      :ensure
      (checked-spec all-specs
                    mandatory-props-ensure
                    optional-props-ensure)))

  (collector/push :ensure doer (spec->resource doer name safe-specs)))

(defn remove
  "Given a file path, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["You must supply exactly one of `:content`, `:from`, `:from-url`, or
    `:from-struct`. If you use `:from-struct` you must also supply
    `:to-format`."
   "The `template-out` and `indoc` macros are useful when specifying :content."
   "`:from` takes a fully-qualified or relative path. If you use the latter,
    Gurp assumes the file is in a ``files/` directory at the same level as the
    directory holding the file being parsed."
   "`:from-struct` and `:to-format` let you turn Janet values into a config
    file. Fully supported file formats are `json`, `toml`, and `yaml`: these
    formats can represent any valid struct. You can create INI files
    (`:to-format \"ini\"`), but the limits of that format mean your struct
    must be a struct of structs, each representing a section. An invalid struct
    will cause an error."
   "Unless you specify TIMESTAMP, only one backup file is kept. Backup files are
    always owned by `root:root`, with mode `0400`."
   "If you try to ensure a file at a path which exists, but is not a file, Gurp
    will error"
   "Gurp can also create key-value pairs (`:to-format \"kvp\"`). It can do this
    from a single-level struct, or from an array. In the latter case, entries
    are alternately keys and values. Using an array lets you create files with
    duplicate keys, which is sometimes necessary."])
