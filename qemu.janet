(def boot-vol "fast/emu-test")

(host "serv"
      (zfs/ensure boot-vol :size "10G")

      (zone/ensure "emu-test"
                   :brand "emu"
                   :autoboot true
                   :recreate 1
                   :image "/var/tmp/braich-151059.raw"

                   (zone/network "emu0"
                                 :allowed-address "192.168.1.102/24"
                                 :global-nic "auto")

                   (zone/attr "type" :value "generic")
                   (zone/attr "diskif" :value "virtio-blk-device")
                   (zone/attr "netif" :value "virtio-net-device")

                   (zone/rctl "zone.cpu-shares"
                              :priv "privileged"
                              :limit 1
                              :action "none")

                   (zone/emu
                     :bios "https://downloads.omnios.org/media/braich/u-boot.bin"
                     :arch "aarch64"
                     :vcpus 4
                     :cpu "cortex-a53"
                     :qemu-args ["-machine virt" "-accel tcg,thread=multi"]
                     :ram "2G"
                     :image-format "raw"
                     :boot-volume boot-vol)))
