(use judge)
(use ../lib/gurp)

(deftest "svcprop-resources"
  (set *collector* (new-collector))

  (setdyn :role-dyn "test-role")

  (svcprop/ensure "mariadb"
                  :properties {:application/datadir "/data"
                               :application/active true
                               :application/timeout 50})
  (svcprop/ensure "mariadb"
                  :property-groups {:application "application"}
                  :properties {:application/datadir "/data"})

  (svcprop/remove "mariadb" :properties ["application/thing"])
  
  (test *collector*
    @{:ensure @{:svcprop @[{:_id "/test-role/svcprop/mariadb"
                            :name "mariadb"
                            :properties {:application/active {:type "boolean" :value true}
                                         :application/datadir {:type "astring" :value "/data"}
                                         :application/timeout {:type "integer" :value 50}}
                            :role "test-role"}
                           {:_id "/test-role/svcprop/mariadb"
                            :name "mariadb"
                            :properties {:application/datadir {:type "astring" :value "/data"}}
                            :property-groups {:application "application"}
                            :role "test-role"}]}
      :remove @{:svcprop @[{:_id "/test-role/svcprop/mariadb"
                            :name "mariadb"
                            :properties ["application/thing"]
                            :role "test-role"}]}}))

(deftest "svcprop-error"
  (test-error
    (svcprop/ensure "mariadb" :wat true)
    "Failed to validate user input for svcprop 'mariadb' : svcprop missing required key(s): properties"))


