(use ./lib)
(import ../collector)

(def doer :pkg)
(def description "Install and uninstall pkg(5) packages.")
(def name-is "Package name, of the form ooce/editor/helix")
(def mandatory-props-ensure {})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a package name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
