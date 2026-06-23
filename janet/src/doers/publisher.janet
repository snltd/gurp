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

(def doer :publisher)
(def description "Add and remove pkg(5) publisher origins.")
(def name-is "Publisher name")
(def mandatory-props-ensure
  {:origin {:types [:array :tuple]
            :help "List of origins, created with publisher/origin "}})
(def optional-props-ensure
  {:mirror {:types [:array :tuple]
            :help "List of mirrors, created with publisher/mirror"}
   :search-first {:types [:boolean]
                  :help "Search this publisher first"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a publisher name and URI, put an ensure struct in the collector"
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

(defn remove
  "Given a publisher name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
