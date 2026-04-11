(use ./lib)
(import ../collector)

(def doer :vnic)
(def description "Manage VNIC objects")
(def name-is "VNIC name")
(def optional-props-ensure
  {:vlan-tag {:types [:number]
              :help "Enable VLAN tagging with the given tag"}
   :with-interface {:types [:boolean]
                    :help "Whether to create an IP interface on the new VNIC"}})
(def mandatory-props-ensure
  {:over {:types [:string]
          :help "Physical link which will serve the VNIC"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:with-interface false})
(def defaults-remove {})

(defn ensure
  "Given a VNIC name and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a VNIC name and spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["VNICs get a random MAC address."
   "If a VNIC exists but has a different VLAN tag or underlying physical NIC to
    the specification, Gurp will try to recreate it."])
