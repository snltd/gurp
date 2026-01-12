(use ./lib)
(import ../collector)

(def doer :group)
(def description "Create and destroy Unix groups.")
(def name-is "Group name")
(def mandatory-ensure-props
  {:gid {:types [:number]
         :help "The group ID"}})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given a group name and GID, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a group name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
