(use judge)
(import ../../../src/doers/zone)

(deftest zone/bhyve
  (test
    (zone/bhyve :ram "3G"
                :vcpus 4
                :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                :boot-volume "tank/bhyve/test"
                :cloudinit-struct {:network {:version 2}})
    {:bhyve @{:acpi false
              :boot-rom "BHYVE_RELEASE"
              :boot-volume "tank/bhyve/test"
              :cloudinit-struct {:network {:version 2}}
              :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
              :ram "3G"
              :vcpus 4
              :wait-for-boot true}})

  (test-error
    (zone/bhyve)
    "In zone/bhyve NO-NAME: did not find mandatory property :ram. Mandatory properties are :ram, :boot-volume, :vcpus")

  (test-error
    (zone/bhyve :ram "3G"
                :vcpus 4
                :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                :boot-volume "tank/bhyve/test"
                :oops "wat?")
    "In zone/bhyve NO-NAME: unexpected property :oops. Valid properties are :ram, :boot-volume, :vcpus, :acpi, :boot-rom, :cloudinit-files, :label, :image-path, :image-format, :wait-for-boot, :cloudinit-struct"))
