(use ./definers)
(import ../collector)

(defdoer :vnic
  "Manage VNIC objects"
  :name-is "VNIC name"
  (ensure
    (property "over" [:string] :mandatory
              "Physical link which will serve the VNIC")
    (property "vlan-tag" [:number] :optional
              "Enable VLAN tagging with the given tag")
    (property "with-interface" [:boolean] :optional
              "Whether to create an IP interface on the new VNIC"))

  (notes
    "VNICs get a random MAC address."
    "If a VNIC exists but has a different VLAN tag or underlying physical NIC to
     the specification, Gurp will try to recreate it."))

(make-fn :ensure :vnic)
(make-fn :remove :vnic)
