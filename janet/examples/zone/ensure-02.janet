(zone/ensure "bhyve-zone"
             :brand "bhyve"
             :autoboot false
             (zone/network "bhyve0"
                           :allowed-address "192.168.1.102/24"
                           :global-nic "auto")
             (zone/bhyve
               :ram "4G"
               :vcpus 4
               :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
               :boot-volume "tank/bhyve/test"
               :cloudinit-struct {:network {:version 2}})
             :dns {:domain "lan.id264.net"
                   :nameservers ["192.168.1.53"
                                 "192.168.1.1"]})
