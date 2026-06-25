(use ./lib)
(import ../collector)

(defdoer :network-flow
  "Manage network flows via flowadm."
  :name-is "Name of flow. Must be unique"

  :mandatory-props-ensure
  {:link {:types [:string]
          :help "NIC/VNIC to which flow applies"}}

  :optional-props-ensure
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

(defensure "network-flow")
(defremove "network-flow")
