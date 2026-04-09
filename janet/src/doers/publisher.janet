(use ./lib)
(import ../collector)

(def doer :publisher)
(def description "Add and remove pkg(5) publisher origins.")
(def name-is "Publisher name")
(def allowed-types ["origin" "mirror"])
(def mandatory-props-ensure
  {:uri {:types [:string]
         :help "Add a pkg publisher with this URI"}
   :type {:types [:string]
          :help (string "Publisher type: one of " (comma-sep allowed-types))}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove
  {:mirror {:types [:string]
            :help "Remove the mirror with the given URI"}})
(def defaults-ensure {:type "origin"})
(def defaults-remove {})

(defn ensure
  "Given a publisher name and URI, put an ensure struct in the collector"
  [name & spec]
  (pinpoint-error :ensure (key-has-value? spec :type allowed-types))
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a publisher name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
