(use ./lib/definers)
(import ../collector)

(defdoer :ip-address
  "Manages IP addresses via ipadm."
  :name-is "Address name, e.g. vnic0/v4"
  (ensure
    (property "type" [:string] :mandatory "Type of address"
              :values ["static" "dhcp"])
    (property "address" [:string] :optional
              "Local IP address with /netmask, if using static address")
    (property "properties" [:struct] :optional
              "Struct of any valid ipadm addrprops")))

(make-fn :ensure :ip-address)
(make-fn :remove :ip-address)
