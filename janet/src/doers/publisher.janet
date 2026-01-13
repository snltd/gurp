(use ./lib)
(import ../collector)

(def doer :publisher)
(def description "Add and remove pkg(5) publisher origins.")
(def name-is "Publisher name")
(def mandatory-ensure-props
  {:uri {:types [:string]
         :help "Add a pkg publiser with this URI"}})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given a publisher name and URI, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a publisher name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
