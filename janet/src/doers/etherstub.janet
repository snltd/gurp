(use ./lib)
(import ../collector)

(def doer :etherstub)
(def description "Create and destroy etherstubs.")
(def name-is "Name of etherstub")
(def mandatory-props-ensure {})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given an etherstub name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an etherstub name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
