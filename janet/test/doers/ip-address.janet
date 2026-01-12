(use judge)
(use ../../src/collector)
(import ../../src/doers/ip-address)

(deftest "ip-address"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (ip-address/ensure "test0/v4"
                     :type "static"
                     :address "192.168.1.13/24"
                     :properties {:prefixlen 24
                                  :transmit true
                                  :private false})

  (ip-address/ensure "test-vnic1/v4"
                     :type "dhcp")

  (ip-address/remove "test-vnic2")

  (test *collector*
    @{:ensure @{:ip-address @[@{:_id "/test-role/ip-address/test0_v4"
                                :address "192.168.1.13/24"
                                :name "test0/v4"
                                :properties {:prefixlen 24
                                             :private false
                                             :transmit true}
                                :role "test-role"
                                :type "static"}
                              @{:_id "/test-role/ip-address/test-vnic1_v4"
                                :name "test-vnic1/v4"
                                :role "test-role"
                                :type "dhcp"}]}
      :remove @{:ip-address @[@{:_id "/test-role/ip-address/test-vnic2"
                                :name "test-vnic2"
                                :role "test-role"}]}}))

(deftest "ip-address-error"
  (test-error
    (ip-address/ensure "bad0" :over "e1000g")
    "did not find mandatory property :type. Mandatory properties are :type"))
