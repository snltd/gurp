(use ../lib)

(defhelper :zone :emu
  "Describe a qemu zone inside a zone resource."

  :optional-props
  {:extra-opts {:types [:table :struct]
                :help "A struct of additional options to pass to the zone"}
   :image-format {:types [:string]
                  :help "Specify the format of the image pointed to by :image-url"}
   :image-path {:types [:string]
                :help "Path to install image - must be raw format"}
   :wait-for-boot {:types [:boolean]
                   :help "Wait for boot, or detach immediately"}}

  :mandatory-props
  {:arch {:types [:string]
          :help "Architecture to emulate"}
   :boot-volume {:types [:string]
                 :help "ZFS boot volume"}
   :cpu {:types [:string]
         :help "CPU model to emulate"}
   :ram {:types [:string]
         :help "Amount of RAM to allocate: e.g. '3G'"}
   :vcpus {:types [:number]
           :help "Number of VCPUs to allocate"}}

  :defaults
  {:wait-for-boot true}

  :notes
  ["in development"])

(defn emu
  "Given a spec, return config for an emu zone"
  [& spec]
  (let [name "NO-NAME"
        spec-struct (make-spec-struct ;spec)
        expanded-spec (spec-with-defaults defaults-emu spec-struct)
        spec-table (pinpoint-error :emu
                                   (checked-spec expanded-spec
                                                 mandatory-props-emu
                                                 optional-props-emu))]

    (struct :emu spec-table)))
