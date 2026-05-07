(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/publisher)

(deftest publisher
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "publisher" (curenv))

  (test *collector*
    @{:ensure @{:publisher @[{:_id "/test-role/publisher/example"
                              :mirror @[@{:name "http://mirror.lan.id264.net"}]
                              :name "example"
                              :origin @[@{:name "http://pkg.lan.id264.net"
                                          :proxy "http://10.2.0.20/1837"}]
                              :role "test-role"}]}
      :remove @{:publisher @[{:_id "/test-role/publisher/old_publisher"
                              :name "old_publisher"
                              :role "test-role"}]}}))

(deftest publisher-error
  (test-error
    (publisher/remove "sysdef" :url "abc")
    "In publisher/remove sysdef: unexpected property :url. Valid properties are :label")

  (test-error
    (publisher/ensure "sysdef"
      :uri "http://pkg.lan.id264.net"
      :type "gibbus")
    "In publisher/ensure sysdef: did not find mandatory property :origin. Mandatory properties are :origin")

  (test-error
    (publisher/ensure "sysdef")
    "In publisher/ensure sysdef: did not find mandatory property :origin. Mandatory properties are :origin"))
