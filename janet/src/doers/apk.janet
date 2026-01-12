(use ./lib)
(import ../collector)

(def doer :apk)
(def description "Install and uninstall APK packages. Only valid in an Alpine LX zone.")
(def name-is "Package name")
(def mandatory-ensure-props {})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-prop-values {})

(defn ensure
  "Given an apk package name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-resource)))

(defn remove
  "Given an apk package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-resource)))
