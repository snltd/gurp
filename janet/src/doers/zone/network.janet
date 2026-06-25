(use ../lib)

(defhelper :zone :network
  "Describe network configuration of a zone resource."
  :name-is "Zone VNIC, which may already exist"

  :mandatory-props
  {:physical {:types [:string]
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

(defn network
  "Given a spec, return a zone network struct."
  [physical & spec]
  (let [name "NO-NAME"
        spec-struct (make-spec-struct :physical physical ;spec)
        expanded-spec (spec-with-defaults defaults-network spec-struct)
        spec-table (pinpoint-error :network
                                   (checked-spec expanded-spec
                                                 mandatory-props-network
                                                 optional-props-network))]

    (struct :net spec-table)))
