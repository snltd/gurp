(use ../lib)

(defhelper :zone :cloudinit
  "Describe cloudinit config inside a zone resource"
  :name-is "cloudinit file name"

  :optional-props
  {:from {:types [:string]
          :help "Copy the given files into the Cloudinit image"}
   :from-struct {:types [:struct :table]
                 :help "Generate a Cloudinit file from the given struct or table"}}

  :mandatory-props
  {:name {:types [:string]
          :help "cloudinit file name. Derived from helper name"}}

  :defaults {}

  :notes ["You must supply exactly one of :from or :from-struct"])

(defn cloudinit
  "Given a spec, return cloudinit config"
  [name & spec]
  (if-not (has-exactly-one-of? [:from :from-struct] spec)
    (pinpoint-error
      :ensure
      (error "need exactly one of :from, :from-struct")))

  (let [spec-struct (make-spec-struct :name name ;spec)
        expanded-spec (spec-with-defaults defaults-cloudinit spec-struct)
        spec-table (pinpoint-error :cloudinit
                                   (checked-spec expanded-spec
                                                 mandatory-props-cloudinit
                                                 optional-props-cloudinit))]

    (struct :cloudinit spec-table)))
