(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/pkgin)

(deftest pkgin
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "pkgin")

  (pkgin/remove "python")

  (test *collector*
        @{:ensure @{:pkgin @[{:_id "/test-role/pkgin/rust"
                              :name "rust"
                              :role "test-role"}]}
          :remove @{:pkgin @[{:_id "/test-role/pkgin/go"
                              :name "go"
                              :role "test-role"}
                             {:_id "/test-role/pkgin/python"
                              :name "python"
                              :role "test-role"}]}}))

(deftest pkgin-error
  (test-error
    (pkgin/remove "go" :version "1.20.1")
    "In pkgin/remove go: unexpected property :version. Valid properties are :label")

  (test-error
    (pkgin/ensure "gurp"
                  :version "1.1.1")
    "In pkgin/ensure gurp: unexpected property :version. Valid properties are :label"))
