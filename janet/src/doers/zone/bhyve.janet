(use ../lib)

(def description-bhyve "Describe a bhyve zone inside a zone resource.")
(def name-is-bhyve "This resource type does not accept a name")
(def defaults-bhyve
  {:wait-for-boot true})

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
  (def spec-struct (make-spec-struct ;spec))
  (def spec-table (checked-spec (spec-with-defaults defaults-bhyve spec-struct)
                                mandatory-props-bhyve
                                optional-props-bhyve))
  (struct :bhyve spec-table))
