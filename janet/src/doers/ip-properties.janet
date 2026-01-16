(use ./lib)
(import ../collector)

(def doer :ip-properties)
(def description "Sets global IP properties, via 'ipadm set-prop'.")
(def name-is "Any convenient name: not used internally")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:properties {:types [:struct]
                :help "A struct whose keys are protocols (e.g. 'ipv4', 'ipv6'),
                      and whose values are structs pairing properties (e.g.
                      :hoplimit, :max_buf) with values"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a protocol and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))
