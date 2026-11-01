(import ../../collector)
(use ../lib/definers)

(defhelper :zone :bhyve
  "Describe a bhyve zone inside a zone resource."
  :unique true
  (helper
    (property "boot-volume" [:string] :mandatory "ZFS boot volume")
    (property "ram" [:string] :mandatory
              "Amount of RAM to allocate: e.g. '3G'")
    (property "vcpus" [:number] :mandatory "Number of VCPUs to allocate")
    (property "boot-rom" [:string] :mandatory "boot ROM image"
              :values ["BHYVE_RELEASE" "BHYVE_RELEASE_CSM"]
              :default "BHYVE_RELEASE")
    (property "wait-for-boot" [:boolean] :mandatory
              "Wait for zone to boot rather than detaching immediately"
              :default true)
    (property "acpi" [:boolean] :optional "whether to enable ACPI in zone")
    (property "image-format" [:string] :optional
              "Specify, rather than guess, the format of the image pointed to
               by `:image-url`")
    (property "image-path" [:string] :optional
              "Path to install image - must be raw format"))

  (notes
    "A bhyve zone must be built from an image. This can be a local path or a URL.
    Specify it with the `:image` key in the main zone config."
    "If your image is a .zst, Gurp will create the `:boot-volume` automatically,
    clobbering it if it already exists. For any other image type, it is the 
    user's responsibility to create the `:boot-volume`."))

(make-helper-fn :bhyve :no-name true)
