(use ./lib)
(import ../collector)

(def doer :etherstub)
(def description "Create and destroy etherstubs.")
(def name-is "Name of etherstub")
(def mandatory-ensure-props {})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given an etherstub name and specification, return an etherstub ensure struct"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an etherstub name and specification, return an etherstub remove struct"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
