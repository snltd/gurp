(use ./lib)
(import ../collector)

(def doer :ip-properties)
(def description "Sets global IP properties, via 'ipadm set-prop'.")
(def name-is "Any convenient name: not used internally")
(def mandatory-props-ensure {})
(def optional-props-ensure protocol-opts)
(def defaults-ensure {})

(defn ensure
  "Given a protocol and spec, put an ensure struct in the collector"
  [name & spec]

  (def spec-table
    (pinpoint-error
      :ensure
      (group-ip-properties mandatory-props-ensure optional-props-ensure ;spec)))

  (collector/push :ensure doer (spec->resource doer name spec-table)))

(def notes
  ["Define `extra_priv_ports` as a comma-separated list."])
