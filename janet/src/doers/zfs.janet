(use ./lib)
(import ../collector)

(def doer :zfs)
(def description "Create, destroy, and modify properties of ZFS filesystems.")
(def name-is "ZFS dataset name")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:properties
   {:types [:struct]
    :help "ZFS properties (:keyword) paired with desired value (:string)"}
   :size
   {:types [:string]
    :help "If specified, creates a ZFS volume of given size (e.g. '10G')"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a dataset name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a dataset name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
