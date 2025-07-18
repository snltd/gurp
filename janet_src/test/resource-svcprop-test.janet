(use judge)
(use ../lib/gurp)

(deftest "test svcprop functions"
  (setdyn :role-dyn "test-role")
  (test
    (svcprop/ensure "mariadb"
                    :properties {:application/datadir "/data"
                                 :application/active true
                                 :application/timeout 50})
    {:svcprop {:_id "/test-role/svcprop/mariadb"
               :action :ensure
               :name "mariadb"
               :properties @{:application/active {:type "boolean" :value true}
                             :application/datadir {:type "astring" :value "/data"}
                             :application/timeout {:type "integer" :value 50}}
               :role "test-role"}})

  (test
    (svcprop/ensure "mariadb"
                    :property-groups {:application "application"}
                    :properties {:application/datadir "/data"})
    {:svcprop {:_id "/test-role/svcprop/mariadb"
               :action :ensure
               :name "mariadb"
               :properties @{:application/datadir {:type "astring" :value "/data"}}
               :property-groups {:application "application"}
               :role "test-role"}}))
