(import ../../collector)
(use ../lib/definers)

(defhelper :zone :dns
  "Define DNS configuration for a native zone."
  (helper
    (property "domain" [:string] :mandatory "Default DNS domain")
    (property "nameservers" [:tuple] :mandatory
              "A list of nameservers. (`:string`)")
    (property "search-list" [:tuple] :optional
              "A list of DNS domains to search. (`:string`)")
    (property "options" [:tuple] :optional
              "A list of resolv.conf options (`:string`)")))

(make-helper-fn :dns :no-name true)
