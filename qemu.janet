(zfs/ensure "rpool/arm64" {:properties {:size "10G"}})

(zone/bhyve
  :vcpus 4
  :ram "8G"
  :image "https://downloads.omnios.org/media/braich/braich-151059.raw.zst"
  :image-format "zst"
  :boot-volume "rpool/arm64")
# :cloudinit-files [(config-file "cloud-init/user-data")]
# :cloudinit-struct
# {:meta-data (cloudinit-meta-data "example-zone")

#  :network-config
#  {:network {:version 2
#             :ethernets
#             {:enp0s6 {:addresses ["192.168.1.55"]
#                       :mtu 1500
#                       :nameservers {:search ["lan.id264.net"]
#                                     :addresses ["192.168.1.53" "1.1.1.1"]}
#                       :routes [{:to "0.0.0.0/0"
#                                 :via "192.168.1.1"}]}}}}})

