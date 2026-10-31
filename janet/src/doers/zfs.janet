(use ./definers)
(import ../collector)

(defn- ensure-empty-mountpoint
  [prop-struct]
  (def prop-table (struct/to-table prop-struct))
  (when (not (has-key? prop-table :mountpoint))
    (put-in prop-table [:mountpoint] "none"))
  (table/to-struct prop-table))

(defdoer :zfs
  "Create, destroy, and modify properties of ZFS filesystems."
  :name-is "ZFS dataset name"
  (ensure
    (property "properties" [:struct] :optional
              "ZFS properties (:keyword) paired with desired value (`:string`)"
              :default {}
              :transform-with ensure-empty-mountpoint)
    (property "size" [:string] :optional
              "If specified, creates a ZFS volume of given size (e.g. '10G')"))

  (notes
    "Gurp does not check parameters are valid, so if you get them wrong the
     first you'll know about it is when you get an error from `zfs(8)`."
    "If you do not set a mountpoint for a filesystem, Gurp will force it to
     'none'."
    "Gurp cannot change the size of an extant volume."
    "`zfs/destroy` is recursive, and will remove all child filesystems and
     snapshots without asking or telling."))

(make-fn :ensure :zfs)
(make-fn :remove :zfs)
