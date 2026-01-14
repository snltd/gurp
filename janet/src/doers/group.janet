(use ./lib)
(import ../collector)

(def doer :group)
(def description "Create and destroy Unix groups.")
(def name-is "Group name")
(def mandatory-props-ensure
  {:gid {:types [:number]
         :help "The group ID"}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a group name and GID, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a group name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
