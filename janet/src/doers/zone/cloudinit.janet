(use ../lib)

(defhelper :zone :cloudinit
  "Describe cloudinit config inside a zone resource"

  :optional-props
  {:files {:types [:tuple]
           :help "Copy the given files into the Cloudinit image"}
   :from-struct {:types [:struct :table]
                 :help "Generate a Cloudinit file from the given struct or
                        table. Top level keys map to files, e.g. 'user-data'"}}

  :mandatory-props {}

  :defaults {}

  :notes [])

(defn cloudinit
  "Given a spec, return cloudinit config"
  [& spec]
  (let [name "NO-NAME"
        spec-struct (make-spec-struct ;spec)
        expanded-spec (spec-with-defaults defaults-cloudinit spec-struct)
        spec-table (pinpoint-error :cloudinit
                                   (checked-spec expanded-spec
                                                 mandatory-props-cloudinit
                                                 optional-props-cloudinit))]

    (struct :cloudinit spec-table)))
