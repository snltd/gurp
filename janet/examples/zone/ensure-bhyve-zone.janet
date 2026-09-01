(zone/ensure "bhyve-zone"
             :brand "bhyve"
             :autoboot false
             :image "/var/tmp/noble-server-cloudimg-amd64.img.raw"

             (zone/network "bhyve0"
                           :allowed-address "10.10.0.2/24"
                           :global-nic "auto")
             (zone/bhyve
               :ram "4G"
               :vcpus 4
               :boot-volume "tank/bhyve/test")

             (zone/cloudinit "meta-data"
                             :from-struct (cloudinit-meta-data "example-zone"))
             (zone/cloudinit "network"
                             :from-struct {:network {:fancy "struct"}})
             (zone/cloudinit "users" :from "cloudwatch/users")
             (zone/cloudinit "packages" :from "cloudwatch/packages"))
