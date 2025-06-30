(use judge)
(use ../lib/gurp)

(deftest "test svc functions"
  (setdyn :role-dyn "test-role")
  (test
    (svc/ensure "important/service"
    :state "enabled")
    {:svc {:_id "/test-role/svc/important_service"
           :action :ensure
           :name "important/service"
           :reloaded-by []
           :restarted-by []
           :role "test-role"
           :state "enabled"}}))
