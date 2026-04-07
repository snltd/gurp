(use ../../src/dsl)

(zone/bhyve
  :vcpus 4
  :ram "8G"
  :image-format "qcow2"
  :boot-volume "tank/byhve/example-boot"
  :cloudinit-files [(config-file "cloud-init/user-data")]
  :cloudinit-struct
  {:meta-data (cloudinit-meta-data "example-zone")

   :network-config
   {:network {:version 2
              :ethernets
              {:enp0s6 {:addresses ["10.10.0.2"]
                        :mtu 1500
                        :nameservers {:search ["localnet"]
                                      :addresses ["10.10.0.53" "1.1.1.1"]}
                        :routes [{:to "0.0.0.0/0"
                                  :via "10.10.0.1"}]}}}}})
