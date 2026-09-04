(use ../lib)

(defhelper :zone :emu
  "Describe a qemu zone inside a zone resource."

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

  :optional-props
  {:bios {:types [:string]
          :help "Path or URL to bios file"}
   :qemu-args {:types [:array :tuple]
                :help "Extra arguments to pass to qemu as `extraN` attrs"}
   :image-format {:types [:string]
                  :help "Specify the format of the image pointed to by :image-url"}
   :image-path {:types [:string]
                :help "Path to install image - must be raw format"}
   :wait-for-boot {:types [:boolean]
                   :help "Wait for boot, or detach immediately"}}

  :defaults
  {:wait-for-boot true}

  :notes
  ["Gurp uses the `extra` attr for the bios filename. If you need to pass more
    flags to qemu, use the `qemu-args` property, or define attrs named extraN
    with N > 1."])

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

      (if-let [extras (spec-table :qemu-args)]
        (set (spec-table :qemu-args) (map safe-val extras)))

    (struct :emu spec-table)))
