(use judge)
(import ../../../src/doers/zone)

(deftest zone/bhyve
  (test
    (zone/bhyve :ram "3G"
                :vcpus 4
                :boot-volume "tank/bhyve/test")
    {:bhyve @{:acpi false
              :boot-rom "BHYVE_RELEASE"
              :boot-volume "tank/bhyve/test"
              :ram "3G"
              :vcpus 4
              :wait-for-boot true}})

  (test-error
    (zone/bhyve)
    "In zone/bhyve NO-NAME: did not find mandatory property :ram. Mandatory properties are :ram, :boot-volume, :vcpus")

  (test-error
    (zone/bhyve :ram "3G"
                :vcpus 4
                :boot-volume "tank/bhyve/test"
                :oops "wat?")
    "In zone/bhyve NO-NAME: unexpected property :oops. Valid properties are :ram, :boot-volume, :vcpus, :boot-rom, :image-path, :label, :image-format, :wait-for-boot, :acpi"))
