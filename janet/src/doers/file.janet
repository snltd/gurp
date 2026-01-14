(use ./lib)
(use ../user-helpers)
(import ../collector)

(def doer :file)
(def description "Create files from multiple sources, or remove them.")
(def name-is "Fully qualified path to file")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:backup-suffix {:types [:string]
                   :help "Back up the file with this suff. Use 'TIMESTAMP' for
                          an epoch timestamp"}
   :from {:types [:string]
          :help "Copy content from this file. If relative, looks in ../files"}
   :from-struct {:types [:struct]
                 :help "Generate a config file from the given struct. Requires
                       :to-format"}
   :from-url {:types [:string]
              :help "Fetch file from the given URL"}
   :group {:types [:string :number]
           :help "The group name or GID of the for this file"}
   :ignore-pattern {:types [:string]
                    :help "When comparing, ignore lines matching this Rust regex"}
   :mode {:types [:string]
          :help "Permissions written as a four-digit octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this file"}
   :to-format {:types [:string]
               :help "Used with :from-struct to try to turn the struct into this
                     format"}
   :with-checksum {:types [:string]
                   :help "Blake3 checksum used to validate files fetched by
                         :from-url"}
   :content {:types [:string]
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
  (string "http://" server-name "/file/" from-path))

(defn ensure
  "Given a file path and spec, put an ensure struct in the collector. If Gurp is
   running as a server, changes local file references into HTTP ones."
  [name & spec]
  (def spec-table (struct/to-table (make-spec-struct ;spec)))

  (if-let [from-path (spec-table :from)]
  (if-let [server-name (dyn :server-name)]
    (do
      (set (spec-table :from) nil)
      (set (spec-table :from-url) (server-url server-name from-path)))
    (let [url-or-qualified-path
          (if (string/find "://" from-path)
            from-path
            (qualify-from-path from-path))]
      (set (spec-table :from) url-or-qualified-path))))

  (def all-specs (spec-with-defaults defaults-ensure spec-table))
  (def safe-specs (checked-spec all-specs
                                mandatory-props-ensure
                                optional-props-ensure))

  (collector/push :ensure doer (spec->resource doer name safe-specs)))

(defn remove
  "Given a file path, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
