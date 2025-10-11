(use judge)
(use ../lib/gurp)

(deftest "vnic-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (vnic/ensure "test-vnic0" :over "e1000g")
  (vnic/remove "test-vnic1")

  (test *collector*
    @{:ensure @{:vnic @[{:_id "/test-role/vnic/test-vnic0"
                         :name "test-vnic0"
                         :over "e1000g"
                         :role "test-role"}]}
      :remove @{:vnic @[{:_id "/test-role/vnic/test-vnic1"
                         :name "test-vnic1"
                         :role "test-role"}]}}))

(deftest "vnic-error"
  (test-error
    (vnic/ensure "missing_link0")
    "Failed to validate user input for vnic 'missing_link0': vnic missing required key(s): over")

  (test-error
    (vnic/ensure "bad_link0" :over "e1000g" :speed 100)
    "Failed to validate user input for vnic 'bad_link0': vnic 'bad_link0' has unrecognised key(s): speed"))
