(use ../lib)

(def doer :zone)
(def description-bhyve "Describe a bhyve zone inside a zone resource.")
(def name-is-bhyve nil)
(def defaults-bhyve
  {:wait-for-boot true
   :boot-rom "BHYVE_RELEASE"
   :acpi false})

(def optional-props-bhyve
  {:cloudinit-struct
   {:types [:struct]
    :help "Generate a Cloudinit file from the given struct. Top level keys map
          to files, e.g. 'user-data'"}
   :cloudinit-files
   {:types [:tuple]
    :help "Copy the given files into the Cloudinit image"}
   :wait-for-boot
   {:types [:boolean]
    :help "Wait for boot, or detach immediately"}
   :image-url
   {:types [:string]
    :help "URL of remote install image"}
   :image-format
   {:types [:string]
    :help "Specify the format of the image pointed to by :image-url"}
   :acpi
   {:types [:boolean]
    :help "whether to enable ACPI in zone"}
   :boot-rom
   {:types [:string]
    :help "boot ROM image: may be BHYVE_RELEASE or BHYVE_RELEASE_CSM"}
   :image-path
   {:types [:string]
    :help "Path to install image - must be raw format"}})

(def mandatory-props-bhyve
  {:ram
   {:types [:string]
    :help "Amount of RAM to allocate: e.g. '3G'"}
   :vcpus
   {:types [:number]
    :help "Number of VCPUs to allocate"}
   :boot-volume
   {:types [:string]
    :help "ZFS boot volume"}})

(defn bhyve
  "Given a spec, return config for a bhyve zone"
  [& spec]
  (def name "NO-NAME")
  (def spec-struct (make-spec-struct ;spec))
  (def expanded-spec (spec-with-defaults defaults-bhyve spec-struct))
  (def spec-table
    (pinpoint-error
      :bhyve
      (checked-spec expanded-spec mandatory-props-bhyve optional-props-bhyve)))
  (struct :bhyve spec-table))

(def notes-bhyve
  ["If your image is a .zst, Gurp will create the `:boot-volume` automatically, clobbering it
    if it already exists. For any other image type, the `:boot-volume` must already exist."
   "You must supply exactly one of `:image-url` and `:image-path`"])
