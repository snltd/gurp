(use ../lib)

(defhelper :zone :network
  "Describe network configuration of a zone resource."
  :name-is "Zone VNIC, which may already exist"

  :mandatory-props
  {:name {:types [:string]
              :help "Zone VNIC. This is the name of the resource, and is not
                     specified with a key"}}

  :optional-props
  {:global-nic {:types [:string]
                :help "Physical NIC on which to create zone VNIC"}
   :allowed-address {:types [:string]
                     :help "IP address, with /netmask"}
   :defrouter {:types [:string]
               :help "IP address of default router"}}

  :defaults
  {:global-nic "auto"})

(make-helper-fn "network")
