(use judge)
(use ../../src/collector)
(import ../../src/doers/vnic)

(deftest vnic
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (vnic/ensure "test-vnic0" :over "e1000g")
  (vnic/remove "test-vnic1")

  (test *collector*
        @{:ensure @{:vnic @[{:_id "/test-role/vnic/test-vnic0"
                             :name "test-vnic0"
                             :over "e1000g"
                             :role "test-role"
                             :with-interface false}]}
          :remove @{:vnic @[{:_id "/test-role/vnic/test-vnic1"
                             :name "test-vnic1"
                             :role "test-role"}]}}))

(deftest vnic-error
  (test-error
    (vnic/ensure "missing_link0")
    "did not find mandatory property :over. Mandatory properties are :over")

  (test-error
    (vnic/ensure "bad_link0" :over "e1000g" :speed 100)
    "unexpected property :speed. Valid properties are :over, :with-interface, :vlan-tag, :label"))
