(use ../lib/definers)
(import ../../collector)
(import ../lib/lib :only [safe-val])

(defhelper :zone :emu
  "Describe a qemu zone inside a zone resource."
  :unique true
  (helper
    (property "arch" [:string] :mandatory "Architecture to emulate")
    (property "boot-volume" [:string] :mandatory "ZFS boot volume")
    (property "cpu" [:string] :mandatory "CPU model to emulate")
    (property "ram" [:string] :mandatory "Amount of RAM to allocate: e.g. '3G'")
    (property "vcpus" [:number] :mandatory "Number of VCPUs to allocate")
    (property "wait-for-boot" [:boolean] :mandatory
              "Wait for the zone to boot, rather than detaching immediately"
              :default true)
    (property "bios" [:string] :optional "Path or URL to bios file")
    (property "qemu-args" [:array :tuple] :optional
              "Extra arguments to pass to qemu as `extraN` attrs"
              :transform-with (partial map lib/safe-val))
    (property "image-format" [:string] :optional
              "Specify the format of the image pointed to by :image-url")
    (property "image-path" [:string] :optional
              "Path to install image - must be raw format"))

  (notes
    "Gurp uses the `extra` attr for the BIOS filename. If you need to pass more
     flags to qemu, use the `qemu-args` property, or define attrs named `extraN`
     with N > 1."))

(make-helper-fn :emu :no-name true)
