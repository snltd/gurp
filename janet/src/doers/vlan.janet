(use ./lib/definers)
(import ../collector)

(defdoer :vlan
  "Manage VLAN objects"
  :name-is "VLAN name"
  (ensure
    (property "over" [:string] :mandatory
              "Physical link which will serve the VLAN")
    (property "vlan-tag" [:number] :mandatory "The VLAN tag ID"))

  (notes
    "You must specify the VLAN link name. Gurp does not support automatic
     naming, as it goes against its policy of assuming nothing."))

(make-fn :ensure :vlan)
(make-fn :remove :vlan)
