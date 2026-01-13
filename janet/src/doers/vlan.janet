(use ./lib)
(import ../collector)

(def doer :vlan)
(def description "Manage VLAN objects")
(def name-is "VLAN name")
(def optional-ensure-props {})
(def mandatory-ensure-props
  {:over {:types [:string]
          :help "Physical link which will serve the VLAN"}
   :vlan-tag {:types [:number]
              :help "The VLAN tag ID"}})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ensure
  "Given a VNIC name ans spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a VNIC name ans spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
