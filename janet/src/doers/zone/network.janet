(use ../lib)

(def doer :zone)
(def description-network "Describe network configuration of a zone resource.")
(def name-is-network "Zone VNIC, which may already exist")
(def mandatory-props-network
  {:physical
   {:types [:string]
    :help "Zone VNIC. This is the name of the resource, and is not specified with a key"}})
(def optional-props-network
  {:global-nic
   {:types [:string]
    :help "Physical NIC on which to create zone VNIC"}
   :allowed-address
   {:types [:string]
    :help "IP address, with /netmask"}
   :defrouter
   {:types [:string]
    :help "IP address of default router"}})
(def defaults-network
  {:global-nic "auto"})

(defn network
  "Given a spec, return a zone network struct."
  [physical & spec]
  (def name "NO-NAME")
  (def spec-struct (make-spec-struct :physical physical ;spec))
  (def expanded-spec (spec-with-defaults defaults-network spec-struct))
  (def spec-table
    (pinpoint-error :network
                    (checked-spec
                      expanded-spec
                      mandatory-props-network
                      optional-props-network)))
  (struct :net spec-table))
