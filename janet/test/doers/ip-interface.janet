(use judge)
(use ../../src/collector)
(use ../../src/doers/ip-interface-protocol)
(import ../../src/doers/ip-interface)

(deftest "ip-interface-protocol"
  (test
  (ip-interface-protocol "ipv4"
      :mtu 1500
      :forwarding true)
    [:protocols
     {"ipv4" {:forwarding true :mtu 1500}}]))
      
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
    @{:ensure @{:ip-interface @[@{:_id "/test-role/ip-interface/test-vnic0"
                                  :name "test-vnic0"
                                  :role "test-role"}
                                @{:_id "/test-role/ip-interface/test-vnic1"
                                  :name "test-vnic1"
                                  :protocols {"ipv4" {:forwarding true :mtu 1500}}
                                  :role "test-role"}]}
      :remove @{:ip-interface @[@{:_id "/test-role/ip-interface/test-vnic3"
                                  :name "test-vnic3"
                                  :role "test-role"}]}}))

(deftest "ip-interface-error"
  (test-error
    (ip-interface/ensure "bad0" :over "e1000g")
    "unexpected property :over. Valid properties are: :protocols, :label"))
