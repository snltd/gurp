(use ./lib)
(import ../collector)

(def doer :vnic)
(def description "Manage VNIC objects")
(def name-is "VNIC name")
(def optional-ensure-props
  {:vlan-tag {:types [:number]
              :help "Enable VLAN tagging with the given tag"}
   :with-interface {:types [:boolean]
                    :help "Whether to create an IP interface on the new VNIC"}})
(def mandatory-ensure-props
  {:over {:types [:string]
          :help "Physical link which will serve the VNIC"}})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values
  {:with-interface false})
(def default-remove-prop-values {})

(defn ensure
  "Given a VNIC name ans spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a VNIC name ans spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
