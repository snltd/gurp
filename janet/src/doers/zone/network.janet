(import ../../collector)
(use ../definers)

(defhelper :zone :network
  "Describe network configuration of a zone resource."
  :name-is "Zone VNIC, which may already exist"
  (helper
    (property "name" [:string] :mandatory
              "Zone VNIC. This is the name of the resource, and is not specified
               with a key")
    (property "global-nic" [:string] :mandatory
              "Physical NIC on which to create zone VNIC"
              :default "auto")
    (property "allowed-address" [:string] :optional "IP address, with /netmask")
    (property "defrouter" [:string] :optional "IP address of default router")))

(make-helper-fn "network")
