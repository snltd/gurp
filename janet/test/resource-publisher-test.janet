(use judge)
(use ../lib/gurp)

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
    "Failed to validate user input for publisher 'sysdef': publisher missing required key(s): uri"))
