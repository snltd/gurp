(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/route)

(deftest route
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "route" (curenv))
  (route/ensure "192.168.1.0/24" :interface "e1000g0")
  (route/ensure "192.168.1.0/24" :interface "router" :force-gateway true)
  (route/remove "192.168.1.1" :gateway "default")

  (test *collector*
        @{:ensure @{:route @[{:_id "/test-role/route/default-gateway"
                              :force-gateway false
                              :gateway "default"
                              :label "default-gateway"
                              :name "192.168.1.1"
                              :role "test-role"}
                             {:_id "/test-role/route/10.0.5.0_24"
                              :flags {:mtu 1500}
                              :force-gateway false
                              :gateway "10.0.5.150"
                              :name "10.0.5.0/24"
                              :role "test-role"}
                             {:_id "/test-role/route/203.0.113.0_24"
                              :force-gateway false
                              :gateway "127.0.0.1"
                              :name "203.0.113.0/24"
                              :role "test-role"
                              :type "blackhole"}
                             {:_id "/test-role/route/192.168.1.0_24"
                              :force-gateway false
                              :interface "e1000g0"
                              :name "192.168.1.0/24"
                              :role "test-role"}
                             {:_id "/test-role/route/192.168.1.0_24"
                              :force-gateway true
                              :interface "router"
                              :name "192.168.1.0/24"
                              :role "test-role"}]}
          :remove @{:route @[{:_id "/test-role/route/10.0.5.0_24"
                              :gateway "10.0.5.150"
                              :name "10.0.5.0/24"
                              :role "test-role"}
                             {:_id "/test-role/route/192.168.1.1"
                              :gateway "default"
                              :name "192.168.1.1"
                              :role "test-role"}]}})

  (test-error
    (route/ensure "192.168.1.1")
    "Provide exactly one of :gateway and :interface")

  (test-error
    (route/ensure "192.168.1.1"
                  :gateway "default"
                  :interface "e1000g")
    "Provide exactly one of :gateway and :interface")

  (test-error
    (route/ensure "192.168.1.1"
                  :gateway "default"
                  :default "1.1.1.1")
    "In route/ensure 192.168.1.1: unexpected property :default. Valid properties are :type, :interface, :force-gateway, :label, :flags, :gateway")

  (test-error
    (route/remove "192.168.1.1")
    "In route/remove 192.168.1.1: did not find mandatory property :gateway. Mandatory properties are :gateway")

  (test-error
    (route/remove "192.168.1.1" :gateway "default" :type "problem")
    "In route/remove 192.168.1.1: unexpected property :type. Valid properties are :gateway, :label"))
