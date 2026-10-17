(use ../lib)

(defhelper :zone :bhyve
  "Describe a bhyve zone inside a zone resource."

  :mandatory-props
  {:boot-volume {:types [:string]
                 :help "ZFS boot volume"}
   :ram {:types [:string]
         :help "Amount of RAM to allocate: e.g. '3G'"}
   :vcpus {:types [:number]
           :help "Number of VCPUs to allocate"}}

  :optional-props
  {:acpi {:types [:boolean]
          :help "whether to enable ACPI in zone"}
   :boot-rom {:types [:string]
              :help "boot ROM image: may be BHYVE_RELEASE or BHYVE_RELEASE_CSM"}
   :image-format {:types [:string]
                  :help "Specify the format of the image pointed to by :image-url"}
   :image-path {:types [:string]
                :help "Path to install image - must be raw format"}
   :wait-for-boot {:types [:boolean]
                   :help "Wait for boot, or detach immediately"}}


  :defaults
  {:acpi false
   :boot-rom "BHYVE_RELEASE"
   :wait-for-boot true}

  :notes
  ["A bhyve zone must be built from an image. This can be a local path or a URL.
    Specify it with the `:image` key in the main zone config."
   "If your image is a .zst, Gurp will create the `:boot-volume` automatically,
    clobbering it if it already exists. For any other image type, it is the 
    user's responsibility to create the `:boot-volume`."])

(make-helper-fn bhyve :without-name true)
