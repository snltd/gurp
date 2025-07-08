(use judge)
(use ../lib/gurp)

(deftest "test zone functions"
  (setdyn :role-dyn "test-role")
  (test
    (zone/ensure "test-zone" )
    {:zone {:_id "/test-role/zone/test-zone"
            :action :ensure
            :autoboot true
            :name "test-zone"
            :role "test-role"}})
  (test
    (zone/remove "defunct-zone")
    {:zone {:_id "/test-role/zone/defunct-zone"
            :action :remove
            :name "defunct-zone"
            :role "test-role"}}))
