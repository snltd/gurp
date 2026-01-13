(use ./lib)
(use ./ip-interface-protocol)
(import ../collector)

(def doer :ip-interface)
(def description "Create and destroy IP interfaces, with optional properties.
                 Properties are supplied with 'ip-interface-protocol'.")
(def name-is "Interface name")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:protocols
   {:types [:struct]
    :help "See 'ip-interface-protocol'"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given an IP interface spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an IP interface spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def description-protocol "Sets IP interface properties for a given protocol.")
(def name-protocol "Protocol. e.g. ipv4, ipv6")
(def mandatory-props-protocol {})
(def optional-props-protocol
  {:properties
   {:types [:struct]
    :help "Struct of ipadm 'ifprop' properties, e.g. :mtu, :forwarding"}})

(defn protocol
  "Given specs, return config for an interface protocol. Key is protocol, values
  are params"
  [protocol & params]
  [:protocols {protocol (struct ;params)}])
