(use judge)
(use ../lib/test-lib)
(import ../../../src/doers/zone)

(deftest zone/bhyve-example
    (test
    (import-test "zone/bhyve-01.janet")
      [:bhyve
       @{:boot-rom "BHYVE_RELEASE"
         :boot-volume "tank/byhve/example-boot"
         :from-helper true
         :image-format "qcow2"
         :ram "8G"
         :unique true
         :vcpus 4
         :wait-for-boot true}]))

(deftest zone/bhyve-static
  (test
    (zone/bhyve :ram "3G"
                :vcpus 4
                :boot-volume "tank/bhyve/test")
    [:bhyve
     @{:boot-rom "BHYVE_RELEASE"
       :boot-volume "tank/bhyve/test"
       :from-helper true
       :ram "3G"
       :unique true
       :vcpus 4
       :wait-for-boot true}])

  (test-error
    (zone/bhyve)
    "In zone/bhyve NO-NAME: did not find mandatory property :ram. Mandatory properties are :boot-rom, :boot-volume, :ram, :vcpus, :wait-for-boot")

  (test-error
    (zone/bhyve :ram "3G"
                :vcpus 4
                :boot-volume "tank/bhyve/test"
                :oops "wat?")
    "In zone/bhyve NO-NAME: unexpected property :oops. Valid properties are :acpi, :boot-rom, :boot-volume, :image-format, :image-path, :label, :ram, :vcpus, :wait-for-boot"))
