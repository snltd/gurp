(use ./definers)
(import ../collector)

(defdoer :network-flow
  "Manage network flows via flowadm."
  :name-is "Name of flow. Must be unique"
  (ensure
    (property "link" [:string] :mandatory "NIC/VNIC to which flow applies")
    (property "local-ip" [:string] :optional
              "Local IP address for flow, optional /mask")
    (property "remote-ip" [:string] :optional
              "RemoteIP address for flow, optional /mask")
    (property "protocol" [:string] :optional "Flow protocol")
    (property "local-port" [:number] :optional "Local port of flow")
    (property "remote-port" [:number] :optional "Remote port of flow")
    (property "dsfield" [:string] :optional "With optional /mask")
    (property "maxbw" [:string] :optional
              "Maximum duplex bandwidth, with K, M or G suffix")
    (property "priority" [:string] :optional "Priority of link"
              :values ["high" "medium" "low"])))

(make-fn :ensure :network-flow)
(make-fn :remove :network-flow)
