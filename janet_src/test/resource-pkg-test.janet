(use judge)
(use ../lib/gurp)

(deftest "pkg-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (pkg/ensure "ooce/editor/rust")
  (pkg/remove "ooce/editor/go")
  (pkg/remove "ooce/editor/python")

  (test *collector*
        @{:ensure @{:pkg @[{:_id "/test-role/pkg/ooce_editor_rust"
                            :name "ooce/editor/rust"
                            :role "test-role"}]}
          :remove @{:pkg @[{:_id "/test-role/pkg/ooce_editor_go"
                            :name "ooce/editor/go"
                            :role "test-role"}
                           {:_id "/test-role/pkg/ooce_editor_python"
                            :name "ooce/editor/python"
                            :role "test-role"}]}}))

(deftest "pkg-error"
  (test-error
    (pkg/ensure "sysdef/tools/gurp"
                :version "1.1.1")
    "Failed to validate user input for pkg 'sysdef/tools/gurp' : pkg 'sysdef/tools/gurp' has unrecognised key(s): version"))

