(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/ip-interface)

(deftest ip-interface
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ip-interface" (curenv))

  (test *collector*
    @{:ensure @{:ip-interface @[{:_id "/test-role/ip-interface/example0"
                                 :name "example0"
                                 :role "test-role"}
                                {:_id "/test-role/ip-interface/example-interface"
                                 :label "example-interface"
                                 :name "example1"
                                 :protocols {:ipv4 {:forwarding true :mtu 1500}
                                             :ipv6 {:forwarding false :mtu 1500}}
                                 :role "test-role"}]}
      :remove @{:ip-interface @[{:_id "/test-role/ip-interface/example2"
                                 :name "example2"
                                 :role "test-role"}]}}))
