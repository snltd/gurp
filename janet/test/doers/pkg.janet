(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/pkg)

(deftest pkg
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "pkg" (curenv))
  (pkg/remove "ooce/developer/python")

  (test *collector*
    @{:ensure @{:pkg @[{:_id "/test-role/pkg/ooce_developer_rust"
                        :name "ooce/developer/rust"
                        :role "test-role"}]}
      :remove @{:pkg @[{:_id "/test-role/pkg/ooce_developer_go"
                        :name "ooce/developer/go"
                        :role "test-role"}
                       {:_id "/test-role/pkg/ooce_developer_python"
                        :name "ooce/developer/python"
                        :role "test-role"}]}}))

(deftest pkg-error
  (test-error
    (pkg/ensure "sysdef/tools/gurp"
                :version "1.1.1")
    "unexpected property :version. Valid properties are :label"))
