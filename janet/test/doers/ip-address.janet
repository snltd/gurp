(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/ip-address)

(deftest ip-address
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ip-address" (curenv)) 

  (test *collector*
        @{:ensure @{:ip-address @[{:_id "/test-role/ip-address/test0_v4"
                                   :address "192.168.1.13/24"
                                   :name "test0/v4"
                                   :properties {:prefixlen 24
                                                :private false
                                                :transmit true}
                                   :role "test-role"
                                   :type "static"}
                                  {:_id "/test-role/ip-address/test-vnic1_v4"
                                   :name "test-vnic1/v4"
                                   :role "test-role"
                                   :type "dhcp"}]}
          :remove @{:ip-address @[{:_id "/test-role/ip-address/test-vnic2"
                                   :name "test-vnic2"
                                   :role "test-role"}]}}))

(deftest ip-address-error
  (test-error
    (ip-address/ensure "bad0" :over "e1000g")
    "did not find mandatory property :type. Mandatory properties are :type"))
