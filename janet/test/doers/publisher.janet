(use judge)
(use ../../src/collector)
(import ../../src/doers/publisher)

(deftest "publisher-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (publisher/ensure "sysdef"
                    :uri "http://pkg.lan.id264.net")

  (publisher/remove "sysdef")

  (test *collector*
        @{:ensure @{:publisher @[{:_id "/test-role/publisher/sysdef"
                                  :name "sysdef"
                                  :role "test-role"
                                  :uri "http://pkg.lan.id264.net"}]}
          :remove @{:publisher @[{:_id "/test-role/publisher/sysdef"
                                  :name "sysdef"
                                  :role "test-role"}]}}))

(deftest "publisher-error"
  (test-error
    (publisher/ensure "sysdef")
    "did not find mandatory property :uri. Mandatory properties are :uri"))
