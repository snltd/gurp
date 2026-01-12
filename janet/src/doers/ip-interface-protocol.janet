(use ./lib)

(def doer :ip-interface-protocol)
(def description "Sets IP interface properties for a given protocol.")
(def name-is "Protocol. e.g. ipv4, ipv6")
(def mandatory-ensure-props {})
(def optional-ensure-props
  {:properties {:types [:struct]
             :help "Struct of ipadm 'ifprop' properties, e.g. :mtu, :forwarding"}})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-ensure-prop-values {})
(def default-remove-prop-values {})

(defn ip-interface-protocol
  "Given specs, return config for an interface protocol. Key is protocol, values
  are params"
  [protocol & params]
  [:protocols {protocol (struct ;params)}])
