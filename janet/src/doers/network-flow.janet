(use ./lib)
(import ../collector)

(def doer :network-flow)
(def description "Manage network flows via flowadm.")
(def name-is "Name of flow. Must be unique")
(def mandatory-props-ensure
  {:link {:types [:string]
          :help "NIC/VNIC to which flow applies"}})
(def optional-props-ensure
  {:local-ip {:types [:string]
              :help "Local IP address for flow, optional /mask"}
   :remote-ip {:types [:string]
               :help "RemoteIP address for flow, optional /mask"}
   :protocol {:types [:string]
              :help "Flow protocol"}
   :local-port {:types [:number]
                :help "Local port of flow"}
   :remote-port {:types [:number]
                 :help "Remote port of flow"}
   :dsfield {:types [:string]
             :help "With optional :mask"}
   :maxbw {:types [:string]
           :help "Maximum duplex bandidth, with K, M or G suffix"}
   :priority {:types [:string]
              :help "Priority of link: high, medium, low"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a name and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a name and spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
