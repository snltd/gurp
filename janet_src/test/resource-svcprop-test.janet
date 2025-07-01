(use judge)
(use ../lib/gurp)

(deftest "test svcprop functions"
  (setdyn :role-dyn "test-role")
  (test
    (svcprop/ensure "mariadb"
                    :application/datadir "/data"
                    :application/active true
                    :application/timeout 50)
    {:svcprop {:_id "/test-role/svcprop/mariadb"
               :action :ensure
               :application/active {:type "boolean" :value true}
               :application/datadir {:type "astring" :value "/data"}
               :application/timeout {:type "integer" :value 50}
               :name "mariadb"
               :role "test-role"}}))
