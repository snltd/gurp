(use ./lib)
(import ../collector)

(def doer :pkg)
(def description "Install and uninstall pkg(5) packages.")
(def name-is "Package name, of the form ooce/editor/helix")
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
