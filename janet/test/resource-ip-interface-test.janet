(use judge)
(use ../lib/gurp)

(deftest "ip-interface-protocol"
  (test
  (ip-interface-protocol "ipv4"
      :mtu 1500
      :forwarding true)
    {"ipv4" {:forwarding true :mtu 1500}}))
      
(deftest "ip-interface-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (ip-interface/ensure "test-vnic0")

  (ip-interface/ensure "test-vnic1"
                       (ip-interface-protocol "ipv6"
                                    :mtu 1500
                                    :forwarding false)
                       (ip-interface-protocol "ipv4"
                                    :mtu 1500
                                    :forwarding true))

  (ip-interface/remove "test-vnic3")

  (test *collector*
    @{:ensure @{:ip-interface @[{:_id "/test-role/ip-interface/test-vnic0"
                                 :name "test-vnic0"
                                 :protocols @{}
                                 :role "test-role"}
                                {:_id "/test-role/ip-interface/test-vnic1"
                                 :name "test-vnic1"
                                 :protocols @{"ipv4" {:forwarding true :mtu 1500}
                                              "ipv6" {:forwarding false :mtu 1500}}
                                 :role "test-role"}]}
      :remove @{:ip-interface @[{:_id "/test-role/ip-interface/test-vnic3"
                                 :name "test-vnic3"
                                 :role "test-role"}]}}))

(deftest "ip-interface-error"
  (test-error
    (ip-interface/ensure "bad0" :over "e1000g")
    "Failed to validate user input for ip-interface 'bad0': ip-interface 'bad0' has unrecognised key(s): over"))
