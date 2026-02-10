(use ./lib)
(import ../collector)

(def doer :apk)
(def description "Install and uninstall APK packages. Only valid in an Alpine LX zone.")
(def name-is "Package name")
(def mandatory-props-ensure {})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given an apk package name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an apk package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def limitations
  ["Only adds and removes packages. You cannot specify or pin package versions."])
