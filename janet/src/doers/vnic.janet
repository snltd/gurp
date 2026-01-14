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
  "Given a VNIC name ans spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a VNIC name ans spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
