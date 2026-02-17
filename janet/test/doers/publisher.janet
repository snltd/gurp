(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/publisher)

(deftest publisher
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "publisher" (curenv))

  (test *collector*
    @{:ensure @{:publisher @[{:_id "/test-role/publisher/new_publisher"
                              :name "new_publisher"
                              :role "test-role"
                              :uri "http://pkg.lan.id264.net"}]}
      :remove @{:publisher @[{:_id "/test-role/publisher/old_publisher"
                              :name "old_publisher"
                              :role "test-role"}]}})

  (test-error
    (publisher/remove "sysdef" :url "abc")
    "In publisher/remove sysdef: unexpected property :url. Valid properties are :label")
    
  (test-error
    (publisher/ensure "sysdef")
    "In publisher/ensure sysdef: did not find mandatory property :uri. Mandatory properties are :uri"))
