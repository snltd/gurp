(use ./lib)
(import ../collector)

(defdoer :ip-address
  "Manages IP addresses via ipadm."
  :name-is "Address name, e.g. vnic0/v4"

  :mandatory-props-ensure
  {:type {:types [:string]
          :help "Type of connection: 'static', 'dhcp'"}}

  :optional-props-ensure
  {:address {:types [:string]
             :help "Local IP address with /netmask, if using static address"}
   :properties {:types [:struct]
                :help "Struct of any valid ipadm addrprops"}})

(defensure "ip-address")
(defremove "ip-address")
