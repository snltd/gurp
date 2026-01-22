(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/publisher)

(deftest publisher
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "publisher" (curenv))

  (test *collector*
        @{:ensure @{:publisher @[{:_id "/test-role/publisher/sysdef"
                                  :name "sysdef"
                                  :role "test-role"
                                  :uri "http://pkg.lan.id264.net"}]}
          :remove @{:publisher @[{:_id "/test-role/publisher/sysdef"
                                  :name "sysdef"
                                  :role "test-role"}]}}))

(deftest publisher-error
  (test-error
    (publisher/ensure "sysdef")
    "did not find mandatory property :uri. Mandatory properties are :uri"))
