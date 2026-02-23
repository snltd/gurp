(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/group)

(deftest group
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "group" (curenv))

  (test *collector*
        @{:ensure @{:group @[{:_id "/test-role/group/new-group"
                              :gid 264
                              :name "new-group"
                              :role "test-role"}]}
          :remove @{:group @[{:_id "/test-role/group/old-group"
                              :name "old-group"
                              :role "test-role"}]}}))

(deftest group-error
  (test-error
    (group/ensure "wat")
    "In group/ensure wat: did not find mandatory property :gid. Mandatory properties are :gid")

  (test-error
    (group/ensure "testusergroup"
                  :gid 264
                  :gecos "Test User")
    "In group/ensure testusergroup: unexpected property :gecos. Valid properties are :gid, :label"))
