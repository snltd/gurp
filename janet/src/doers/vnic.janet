(use ./lib)
(import ../collector)

(defdoer :vnic
  "Manage VNIC objects"
  :name-is "VNIC name"

  :optional-props-ensure
  {:vlan-tag {:types [:number]
              :help "Enable VLAN tagging with the given tag"}
   :with-interface {:types [:boolean]
                    :help "Whether to create an IP interface on the new VNIC"}}

  :mandatory-props-ensure
  {:over {:types [:string]
          :help "Physical link which will serve the VNIC"}}

  :defaults-ensure
  {:with-interface false}

  :notes
  ["VNICs get a random MAC address."
   "If a VNIC exists but has a different VLAN tag or underlying physical NIC to
    the specification, Gurp will try to recreate it."])

(defensure "vnic")
(defremove "vnic")
