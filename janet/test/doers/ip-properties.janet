(use judge)
(use ../../src/collector)
(import ../../src/doers/ip-properties)

(deftest ip-properties
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (ip-properties/ensure "general"
                        :ipv6 {:hoplimit 123
                               :hostmodel "weak"}
                        :ipv4 {:hostmodel "weak"}
                        :icmp {:max_buf 1234567})

  (test *collector*
        @{:ensure @{:ip-properties @[{:_id "/test-role/ip-properties/general"
                                      :protocols {:icmp {:max_buf 1234567}
                                                  :ipv4 {:hostmodel "weak"}
                                                  :ipv6 {:hoplimit 123 :hostmodel "weak"}}
                                      :name "general"
                                      :role "test-role"}]}
          :remove @{}}))

(deftest ip-properties-error
  (test-error
    (ip-properties/ensure "general"
                          :ipv6 [1234567])
    "ipv6 is of type :tuple. Allowed types :struct, :table")

  (test-error
    (ip-properties/ensure "general"
                          :max-buf 1234567)
    "unexpected property :max-buf. Valid properties are :ipv6, :ipv4, :ip, :label, :udp, :icmp, :tcp, :sctp"))
