(use ./lib)
(import ../collector)
(import ./publisher/origin :prefix "" :export true)
(import ./publisher/mirror :prefix "" :export true)

# publisher
#   ├── sticky (bool)
#   ├── enabled (bool)
#   ├── search-first / search-before / search-after (ordering)
#   ├── ssl-key, ssl-cert
#   └── origins[]
#         ├── uri
#         └── proxy (optional, per-URI)
#       mirrors[]
#         ├── uri
#         └── proxy (optional, per-URI) 

(defdoer :publisher
  "Add and remove pkg(5) publisher origins."
  :name-is "Publisher name"

  :mandatory-props-ensure
  {:origin {:types [:array :tuple]
            :help "List of origins, created with publisher/origin "}}

  :optional-props-ensure
  {:mirror {:types [:array :tuple]
            :help "List of mirrors, created with publisher/mirror"}
   :search-first {:types [:boolean]
                  :help "Search this publisher first"}})

(defn ensure
  [name & spec]
  (var modified-spec spec)
  (expand-resource :origin)
  (expand-resource :mirror)

  (let [modified-spec-struct (make-spec-struct ;modified-spec)
        spec-struct (pinpoint-error
                      :ensure
                      (checked-spec
                        modified-spec-struct
                        mandatory-props-ensure
                        optional-props-ensure))
        spec-table (spec-with-defaults defaults-ensure spec-struct)]

    (collector/push :ensure doer (spec->resource doer name spec-table))))

(defremove "publisher")
