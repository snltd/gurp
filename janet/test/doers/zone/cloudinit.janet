(use judge)
(use ../test-lib)
(import ../../../src/doers/zone)

(deftest zone/cloudinit
  (setdyn :gurp-config-root "/gurpdir")
  (test
    (import-test "zone/cloudinit.janet")
    [:cloudinit
     @{:from-helper true
       :from-struct {:network {:ethernets {:enp0s6 {:addresses ["10.10.0.2"]
                                                    :mtu 1500
                                                    :nameservers {:addresses ["10.10.0.53" "1.1.1.1"]
                                                                  :search ["localnet"]}
                                                    :routes [{:to "0.0.0.0/0" :via "10.10.0.1"}]}}
                               :version 2}}
       :name "network-config"
       :unique false}])

  (test
    (zone/cloudinit "user-data" :from "user-data")
    [:cloudinit
     @{:from "/gurpdir/files/user-data"
       :from-helper true
       :name "user-data"
       :unique false}])

  (test-error
    (zone/cloudinit "user-data" :from "user-data" :from-struct {:network {}})
    "In zone/cloudinit user-data: need exactly one of :from, :from-struct")

  (test-error
    (zone/cloudinit "user-data" :from "file" :oops "file")
    "In zone/cloudinit user-data: unexpected property :oops. Valid properties are :from, :from-struct, :label")

  (test-error
    (zone/cloudinit "user-data")
    "In zone/cloudinit user-data: need exactly one of :from, :from-struct"))
