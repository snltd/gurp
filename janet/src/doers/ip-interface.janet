(use ./lib)
(import ../collector)

(def doer :ip-interface)
(def description "Create and destroy IP interfaces, with optional properties.
                 Properties are supplied with 'ip-interface-protocol'.")
(def name-is "Interface name")
(def mandatory-props-ensure {})
(def optional-props-ensure protocol-opts)
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given an IP interface name and spec, put an ensure struct in the collector"
  [name & spec]

  (def spec-table
    (group-ip-properties mandatory-props-ensure optional-props-ensure ;spec))

  (collector/push :ensure doer (spec->resource doer name spec-table)))

(defn remove
  "Given an IP interface spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
