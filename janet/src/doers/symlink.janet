(use ./lib)
(import ../collector)

(def doer :symlink)
(def description "Create and remove symbolic links.")
(def name-is "Qualified path to the link that will be created")
(def mandatory-ensure-props
  {:source {:types [:string]
            :help "The file the symlink points to"}})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given a symlink path and target, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a symlink path, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
