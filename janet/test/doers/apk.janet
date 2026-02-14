(use judge)
(use ../../src/collector)
(use ./test-lib)
(import ../../src/doers/apk)

(deftest apk
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "apk" (curenv))

  (apk/remove "python")

  (test *collector*
        @{:ensure @{:apk @[{:_id "/test-role/apk/rust"
                            :name "rust"
                            :role "test-role"}]}
          :remove @{:apk @[{:_id "/test-role/apk/go"
                            :name "go"
                            :role "test-role"}
                           {:_id "/test-role/apk/python"
                            :name "python"
                            :role "test-role"}]}}))

(deftest apk-error
  (test-error
    (apk/ensure "gurp"
                :version "1.1.1")
    "unexpected property :version. Valid properties are :label"))
