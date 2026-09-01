(use judge)
(use ../test-lib)
(import ../../../src/doers/zone)

(deftest zone/cloudinit
  (test
    (import-test "zone/cloudinit.janet")
    {:cloudinit @{:from-struct {:network {:ethernets {:enp0s6 {:addresses ["10.10.0.2"]
                                                               :mtu 1500
                                                               :nameservers {:addresses ["10.10.0.53" "1.1.1.1"]
                                                                             :search ["localnet"]}
                                                               :routes [{:to "0.0.0.0/0" :via "10.10.0.1"}]}}
                                          :version 2}}
                  :name "network-config"}})

  (test
    (zone/cloudinit "user-data" :from "user-data")
    {:cloudinit @{:from "user-data" :name "user-data"}})

  (test-error
    (zone/cloudinit "user-data" :from "user-data" :from-struct {:network {}})
    "In zone/ensure user-data: need exactly one of :from, :from-struct")

  (test-error
    (zone/cloudinit "user-data" :from "file" :oops "file")
    "In zone/cloudinit user-data: unexpected property :oops. Valid properties are :name, :from-struct, :from, :label")

  (test-error
    (zone/cloudinit "user-data")
    "In zone/ensure user-data: need exactly one of :from, :from-struct"))
