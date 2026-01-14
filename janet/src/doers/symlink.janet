(use ./lib)
(import ../collector)

(def doer :symlink)
(def description "Create and remove symbolic links.")
(def name-is "Qualified path to the link that will be created")
(def mandatory-props-ensure
  {:source {:types [:string]
            :help "The file the symlink points to"}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a symlink path and target, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a symlink path, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
