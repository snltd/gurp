(use ../lib)

(defhelper :zone :bhyve
  "Describe a bhyve zone inside a zone resource."

  :mandatory-props
  {:boot-volume {:types [:string]
                 :help "ZFS boot volume"}
   :ram {:types [:string]
         :help "Amount of RAM to allocate: e.g. '3G'"}
   :vcpus {:types [:number]
           :help "Number of VCPUs to allocate"}
   :boot-rom {:types [:string]
              :help "boot ROM image"
              :values ["BHYVE_RELEASE" "BHYVE_RELEASE_CSM"]
              :default "BHYVE_RELEASE"}
   :wait-for-boot {:types [:boolean]
                   :help "Wait for zone to boot rather than detaching
                          immediately"
                   :default true}}

  :optional-props
  {:acpi {:types [:boolean]
          :help "whether to enable ACPI in zone"}
   :image-format {:types [:string]
                  :help "Specify, rather than guess, the format of the image
                         pointed to by `:image-url`"}
   :image-path {:types [:string]
                :help "Path to install image - must be raw format"}}

  :notes
  ["A bhyve zone must be built from an image. This can be a local path or a URL.
    Specify it with the `:image` key in the main zone config."
   "If your image is a .zst, Gurp will create the `:boot-volume` automatically,
    clobbering it if it already exists. For any other image type, it is the 
    user's responsibility to create the `:boot-volume`."])

(make-helper-fn bhyve :without-name true)
