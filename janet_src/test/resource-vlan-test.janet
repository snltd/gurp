(use judge)
(use ../lib/gurp)

(deftest "vlan-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (vlan/ensure "e1000g1000"
               :over "e1000g0"
               :vlan-tag 10)

  (vlan/ensure "testvlan2"
               :over "bge0"
               :vlan-tag 20)

  (vlan/remove "old-vlan")

  (test *collector*
    @{:ensure @{:vlan @[{:_id "/test-role/vlan/e1000g1000"
                         :name "e1000g1000"
                         :over "e1000g0"
                         :role "test-role"
                         :vlan-tag 10}
                        {:_id "/test-role/vlan/testvlan2"
                         :name "testvlan2"
                         :over "bge0"
                         :role "test-role"
                         :vlan-tag 20}]}
      :remove @{:vlan @[{:_id "/test-role/vlan/old-vlan"
                         :name "old-vlan"
                         :role "test-role"}]}})

  (deftest "vlan-error"
    (test-error
      (vlan/ensure "test-vlan-1")
      "Failed to validate user input for vlan 'test-vlan-1': vlan missing required key(s): over, vlan-tag")

    (test-error
      (vlan/ensure "test-vlan"
                   :over "e1000g0"
                   :vlan-tag 24
                   :with "field")
      "Failed to validate user input for vlan 'test-vlan': vlan 'test-vlan' has unrecognised key(s): with")))
