(use judge)
(use ../../src/collector)
(import ../../src/doers/ip-properties)

(deftest "ip-properties"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (ip-properties/ensure "general"
                        :properties {:ipv6 {:hoplimit 123
                                            :hostmodel "weak"}
                                     :ipv4 {:hostmodel "weak"}
                                     :icmp {:max_buf 1234567}})

  (test *collector*
    @{:ensure @{:ip-properties @[@{:_id "/test-role/ip-properties/general"
                                   :name "general"
                                   :properties {:icmp {:max_buf 1234567}
                                                :ipv4 {:hostmodel "weak"}
                                                :ipv6 {:hoplimit 123 :hostmodel "weak"}}
                                   :role "test-role"}]}
      :remove @{}}))
