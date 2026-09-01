(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/ip-interface)

(deftest ip-interface
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ip-interface")

  (test *collector*
    @{:ensure @{:ip-interface @[{:_id "/test-role/ip-interface/example-interface"
                                 :label "example-interface"
                                 :name "example1"
                                 :protocols {:ipv4 {:forwarding true :mtu 1500}
                                             :ipv6 {:forwarding false :mtu 1500}}
                                 :role "test-role"}
                                {:_id "/test-role/ip-interface/example0"
                                 :name "example0"
                                 :role "test-role"}]}
      :remove @{:ip-interface @[{:_id "/test-role/ip-interface/example2"
                                 :name "example2"
                                 :role "test-role"}]}}))

(deftest ip-address-error
  (test-error
    (ip-interface/ensure "bad0" :ipv5 {:forwarding true})
    "In ip-interface/ensure bad0: unexpected property :ipv5. Valid properties are :ipv6, :ipv4, :ip, :label, :udp, :icmp, :tcp, :sctp")

  (test-error
    (ip-interface/ensure "bad1" :ipv4 "yay!")
    "In ip-interface/ensure bad1: ipv4 is of type :string. Allowed types :struct, :table")

  (test-error
    (ip-interface/remove "bad2" :ipv4 {:forwarding true})
    "In ip-interface/remove bad2: unexpected property :ipv4. Valid properties are :label"))
