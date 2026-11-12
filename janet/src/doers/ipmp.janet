(use ./lib/definers)
(import ../collector)

(defdoer :ipmp
  "Manage IPMP interfaces"
  :name-is "IPMP interface name"
  (ensure
    (property "interfaces" [:tuple] :mandatory
              "Interfaces which should form IPMP group"))

  (notes
    "`ipmp/ensure` assumes that the interfaces it is given already exist. It is
     your responsibility to create them, with the `ip-interface` doer"))

(make-fn :ensure "ipmp")
(make-fn :remove "ipmp")
