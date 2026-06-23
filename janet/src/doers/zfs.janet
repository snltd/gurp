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
  (let [spec-table (struct/to-table (make-spec-struct ;spec))]
    (if-not (has-key? spec-table :properties)
      (set (spec-table :properties) {}))

    (when
      (and (not (has-key? spec-table :size))
           (not (get-in spec-table [:properties :mountpoint])))
      (set [spec-table :properties] (struct/to-table (spec-table :properties)))
      (put-in spec-table [:properties :mountpoint] "none")
      (set [spec-table :properties] (table/to-struct (spec-table :properties))))

    (let [all-specs (spec-with-defaults defaults-ensure spec-table)
          safe-specs (pinpoint-error
                       :ensure
                       (checked-spec all-specs
                                     mandatory-props-ensure
                                     optional-props-ensure))]

      (collector/push :ensure doer (spec->resource doer name safe-specs)))))

(defn remove
  "Given a dataset name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["Gurp does not check parameters are valid, so if you get them wrong the
    first you'll know about it is when you get an error from `zfs(8)`."
   "If you do not set a mountpoint for a filesystem, Gurp will force it to
    'none'."
   "Gurp cannot change the size of an extant volume."
   "zfs/destroy is recursive, and will remove all child filesystems and
    snapshots without asking or telling."])
