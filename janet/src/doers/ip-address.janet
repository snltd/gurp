(use ./lib)
(import ../collector)

(def doer :ip-address)
(def description "Manages IP addresses via ipadm.")
(def name-is "Address name, e.g. vnic0/v4")
(def mandatory-props-ensure
  {:type {:types [:string]
          :help "Type of connection: 'static', 'dhcp'"}})
(def optional-props-ensure
  {:address {:types [:string]
             :help "Local IP address with /netmask, if using static address"}
   :properties {:types [:struct]
                :help "Struct of any valid ipadm addrprops"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given an IP address spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an IP address spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
