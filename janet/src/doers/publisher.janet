(use ./lib)
(import ../collector)

(def doer :publisher)
(def description "Add and remove pkg(5) publisher origins.")
(def name-is "Publisher name")
(def mandatory-props-ensure
  {:uri {:types [:string]
         :help "Add a pkg publiser with this URI"}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a publisher name and URI, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a publisher name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
