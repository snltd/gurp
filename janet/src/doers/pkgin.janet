(use ./lib)
(import ../collector)

(def doer :pkgin)
(def description "Install and uninstall pkgin packages. Only valid in a pkgsrc
                 zone.")
(def name-is "Package name")
(def mandatory-ensure-props {})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given a package name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
