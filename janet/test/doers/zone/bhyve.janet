(use judge)
(import ../../../src/doers/zone)

(deftest zone/bhyve
  (test
    (zone/bhyve :ram "3G"
                :vcpus 4
                :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                :boot-volume "tank/bhyve/test"
                :cloudinit-struct {:network {:version 2}})
    {:bhyve @{:boot-volume "tank/bhyve/test"
              :cloudinit-struct {:network {:version 2}}
              :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
              :ram "3G"
              :vcpus 4
              :wait-for-boot true}})

  (test-error
    (zone/bhyve)
    "did not find mandatory property :ram. Mandatory properties are :ram, :boot-volume, :vcpus")

  (test-error
    (zone/bhyve :ram "3G"
                :vcpus 4
                :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                :boot-volume "tank/bhyve/test"
                :oops "wat?")
    "unexpected property :oops. Valid properties are :ram, :boot-volume, :vcpus, :cloudinit-files, :image-path, :label, :image-url, :image-format, :wait-for-boot, :cloudinit-struct"))
