(use judge)
(use ../test-lib)
(import ../../../src/doers/zone)

(deftest zone/bhyve-example
    (test
    (import-test "zone/bhyve-01.janet")
      {:bhyve @{:acpi false
                :boot-rom "BHYVE_RELEASE"
                :boot-volume "tank/byhve/example-boot"
                :image-format "qcow2"
                :ram "8G"
                :vcpus 4
                :wait-for-boot true}}))

(deftest zone/bhyve-static
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
