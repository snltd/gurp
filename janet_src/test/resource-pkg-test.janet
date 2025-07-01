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
  (test-error
    (pkg/ensure "sysdef/tools/gurp"
                :version "1.1.1")
    "pkg 'sysdef/tools/gurp' has unrecognised key(s): version")

  (test
    (pkg/remove "ooce/editor/helix")
    {:pkg {:_id "/test-role/pkg/ooce_editor_helix"
           :action :remove
           :name "ooce/editor/helix"
           :role "test-role"}}))
