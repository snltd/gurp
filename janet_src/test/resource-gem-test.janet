(use judge)
(use ../lib/gurp)

(deftest "test gem functions"
  (setdyn :role-dyn "test-role")
  (test
    (gem/ensure "wavefront-cli")
    {:gem {:_id "/test-role/gem/wavefront-cli"
           :action :ensure
           :name "wavefront-cli"
           :role "test-role"}})
  (test
    (gem/remove "webscale")
    {:gem {:_id "/test-role/gem/webscale"
           :action :remove
           :name "webscale"
           :role "test-role"}}))
  
    
