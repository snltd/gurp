(use ./lib)
(import ../collector)

(defdoer :vlan
  "Manage VLAN objects"
  :name-is "VLAN name"

  :mandatory-props-ensure
  {:over {:types [:string]
          :help "Physical link which will serve the VLAN"}
   :vlan-tag {:types [:number]
              :help "The VLAN tag ID"}}

  :notes
  ["You must specify the VLAN link name. Gurp does not support automatic naming,
    as it goes against its policy of assuming nothing."])

(defensure "vlan")
(defremove "vlan")
