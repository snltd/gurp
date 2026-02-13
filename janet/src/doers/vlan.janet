(use ./lib)
(import ../collector)

(def doer :vlan)
(def description "Manage VLAN objects")
(def name-is "VLAN name")
(def optional-props-ensure {})
(def mandatory-props-ensure
  {:over {:types [:string]
          :help "Physical link which will serve the VLAN"}
   :vlan-tag {:types [:number]
              :help "The VLAN tag ID"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a VNIC name ans spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a VNIC name ans spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["You must specify the VLAN link name. Gurp does not support automatic naming,
    as it goes against its policy of assuming nothing."])
