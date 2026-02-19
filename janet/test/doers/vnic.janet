(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/vnic)

(deftest vnic
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "vnic" (curenv))

  (test *collector*
        @{:ensure @{:vnic @[{:_id "/test-role/vnic/vnic0"
                             :name "vnic0"
                             :over "e1000g"
                             :role "test-role"
                             :with-interface false}
                            {:_id "/test-role/vnic/vnic1"
                             :name "vnic1"
                             :over "e1000g"
                             :role "test-role"
                             :vlan-tag 10
                             :with-interface true}]}
          :remove @{:vnic @[{:_id "/test-role/vnic/vnic2"
                             :name "vnic2"
                             :role "test-role"}]}})

  (test-error
    (vnic/ensure "missing_link0")
    "In vnic/ensure missing_link0: did not find mandatory property :over. Mandatory properties are :over")

  (test-error
    (vnic/ensure "bad_link0" :over "e1000g" :speed 100)
    "In vnic/ensure bad_link0: unexpected property :speed. Valid properties are :over, :with-interface, :vlan-tag, :label"))
