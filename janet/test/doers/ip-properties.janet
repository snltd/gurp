(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/ip-properties)

(deftest ip-properties
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ip-properties" (curenv))

  (test *collector*
    @{:ensure @{:ip-properties @[{:_id "/test-role/ip-properties/general"
                                  :name "general"
                                  :protocols {:icmp {:max_buf 262000}
                                              :ipv4 {:forwarding true}
                                              :ipv6 {:hoplimit 250}
                                              :sctp {:max_buf 1048000}
                                              :tcp {:sack "passive"}
                                              :udp {:extra_priv_ports "2050,4040"}}
                                  :role "test-role"}]}
      :remove @{}}))

(deftest ip-properties-error
  (test-error
    (ip-properties/ensure "general"
                          :ipv6 [1234567])
    "In ip-properties/ensure general: ipv6 is of type :tuple. Allowed types :struct, :table")

  (test-error
    (ip-properties/ensure "general"
                          :max-buf 1234567)
    "In ip-properties/ensure general: unexpected property :max-buf. Valid properties are :ipv6, :ipv4, :ip, :label, :udp, :icmp, :tcp, :sctp"))
