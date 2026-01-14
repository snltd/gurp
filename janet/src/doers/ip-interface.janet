(use ./lib)
(import ../collector)

(def doer :ip-interface)
(def description "Create and destroy IP interfaces, with optional properties.
                 Properties are supplied with 'ip-interface-protocol'.")
(def name-is "Interface name")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:protocols
   {:types [:struct :table]
    :help "See 'ip-interface-protocol'"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  [name & temp-spec]
  (def protocols @{})
  (def spec @[])

  (loop [[prop-name prop-value] :in (partition 2 (flatten temp-spec))]
    (if (= (type prop-value) :struct) # making a big assumption here!
      (set (protocols prop-name) prop-value)
      (array/concat spec [prop-name prop-value])))

  (if-not (empty? protocols)
    (array/concat spec [:protocols protocols]))

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
  "Given specs, return config for an interface protocol."
  [protocol & params]
  [protocol (make-spec-struct ;params)])
