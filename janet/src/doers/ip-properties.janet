(use ./lib)
(import ../collector)

(def doer :ip-properties)
(def description "Sets global IP properties, via 'ipadm set-prop'.")
(def name-is "Any convenient name: not used internally")
(def mandatory-ensure-props {})
(def optional-ensure-props
  {:properties {:types [:struct]
                :help "A struct whose keys are protocols (e.g. 'ipv4', 'ipv6'),
                      and whose values are structs pairing properties (e.g.
                      :hoplimit, :max_buf) with values"}})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given a protocol and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))
