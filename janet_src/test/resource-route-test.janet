(use judge)
(use ../lib/gurp)

(deftest "route-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (route/ensure "10.0.5.0/24"
                :gateway "10.0.5.150"
                :flags {:mtu 1500
                        :interface true})
  (route/ensure "192.168.1.1" :gateway "default")
  (route/remove "10.0.5.0/24" :gateway "10.0.5.150")
  (route/remove "192.168.1.1" :gateway "default")

  (test *collector*
    @{:ensure @{:route @[{:_id "/test-role/route/10.0.5.0_24"
                          :flags {:interface true :mtu 1500}
                          :gateway "10.0.5.150"
                          :name "10.0.5.0/24"
                          :role "test-role"}
                         {:_id "/test-role/route/192.168.1.1"
                          :gateway "default"
                          :name "192.168.1.1"
                          :role "test-role"}]}
      :remove @{:route @[{:_id "/test-role/route/10.0.5.0_24"
                          :gateway "10.0.5.150"
                          :name "10.0.5.0/24"
                          :role "test-role"}
                         {:_id "/test-role/route/192.168.1.1"
                          :gateway "default"
                          :name "192.168.1.1"
                          :role "test-role"}]}}))

(deftest "route-error"
  (test-error
    (route/ensure "192.168.1.1"
                  :gateway "default"
                  :default "1.1.1.1")
    "Failed to validate user input for route '192.168.1.1': route '192.168.1.1' has unrecognised key(s): default")

  (test-error
    (route/ensure "192.168.1.1")
    "Failed to validate user input for route '192.168.1.1': route missing required key(s): gateway"))
