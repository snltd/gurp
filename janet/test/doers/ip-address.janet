(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/ip-address)

(deftest ip-address
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ip-address")

  (test *collector*
    @{:ensure @{:ip-address @[{:_id "/test-role/ip-address/example1_v4"
                               :name "example1/v4"
                               :role "test-role"
                               :type "dhcp"}
                              {:_id "/test-role/ip-address/example0_v4"
                               :address "192.168.1.13/24"
                               :name "example0/v4"
                               :properties {:prefixlen 24
                                            :private false
                                            :transmit true}
                               :role "test-role"
                               :type "static"}]}
      :remove @{:ip-address @[{:_id "/test-role/ip-address/example2_v4"
                               :name "example2/v4"
                               :role "test-role"}]}}))

(deftest ip-address-error
  (test-error
    (ip-address/ensure "bad0" :over "e1000g")
    "In ip-address/ensure bad0: did not find mandatory property :type. Mandatory properties are :type"))
