(use ../lib)

(defhelper :zone :network
  "Describe network configuration of a zone resource."
  :name-is "Zone VNIC, which may already exist"

  :mandatory-props
  {:name {:types [:string]
          :help "Zone VNIC. This is the name of the resource, and is not
                     specified with a key"}
   :global-nic {:types [:string]
                :help "Physical NIC on which to create zone VNIC"
                :default "auto"}}

  :optional-props
  {:allowed-address {:types [:string]
                     :help "IP address, with /netmask"}
   :defrouter {:types [:string]
               :help "IP address of default router"}})

(make-helper-fn "network")
