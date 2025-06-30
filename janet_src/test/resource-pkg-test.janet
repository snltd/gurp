(use judge)
(use ../lib/gurp)

(deftest "test pkg functions"
  (setdyn :role-dyn "test-role")
  (test
    (pkg/ensure "ooce/editor/helix")
    {:pkg {:_id "/test-role/pkg/ooce_editor_helix"
           :action :ensure
           :name "ooce/editor/helix"
           :role "test-role"}})
  (test
    (pkg/remove "ooce/editor/helix")
    {:pkg {:_id "/test-role/pkg/ooce_editor_helix"
           :action :remove
           :name "ooce/editor/helix"
           :role "test-role"}}))
  
    
