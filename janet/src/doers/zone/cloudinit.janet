(use ../lib)

(defhelper :zone :cloudinit
  "Describe cloudinit config inside a zone resource"
  :name-is "cloudinit file name"
  :exactly-one-of [:from :from-struct]

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

(make-helper-fn "cloudinit")
