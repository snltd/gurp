(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/vlan)

(deftest vlan
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "vlan" (curenv))

  (test *collector*
    @{:ensure @{:vlan @[{:_id "/test-role/vlan/e1000g010"
                         :name "e1000g010"
                         :over "e1000g0"
                         :role "test-role"
                         :vlan-tag 10}]}
      :remove @{:vlan @[{:_id "/test-role/vlan/old-vlan"
                         :name "old-vlan"
                         :role "test-role"}]}}))

(deftest vlan-error
  (test-error
    (vlan/ensure "test-vlan-1")
    "did not find mandatory property :over. Mandatory properties are :over, :vlan-tag")

  (test-error
    (vlan/ensure "test-vlan"
                 :over "e1000g0"
                 :vlan-tag 24
                 :with "field")
    "unexpected property :with. Valid properties are :over, :vlan-tag, :label"))
