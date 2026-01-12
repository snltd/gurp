(use ./lib)
(use ./ip-interface-protocol)
(import ../collector)

(def doer :ip-interface)
   (def description "Create and destroy IP interfaces, with optional properties. Properties
                 are supplied with 'ip-interface-protocol'.")
(def name-is "Interface name")
(def mandatory-ensure-props {})
(def optional-ensure-props
  {:protocols {:types [:struct]
             :help "See 'ip-interface-protocol'"}})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given an IP interface spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an IP interface spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
