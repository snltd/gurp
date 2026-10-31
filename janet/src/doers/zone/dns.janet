(use ../lib)

(defhelper :zone :dns
  "Define DNS configuration for a native zone."

  :mandatory-props
  {:domain {:types [:string]
            :help "Default DNS domain"}
   :nameservers {:types [:tuple]
                 :help "A list of nameservers. (`:string`)"}}

  :optional-props
  {:search-list {:types [:tuple]
                 :help "A list of DNS domains to search. (`:string`)"}
   :options {:types [:tuple]
             :help "A list of resolv.conf options (`:string`)"}})

(make-helper-fn dns :without-name true)
