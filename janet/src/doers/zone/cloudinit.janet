(use ../lib)

(defhelper :zone :cloudinit
  "Describe cloudinit config inside a zone resource"
  :name-is "cloudinit file name"
  :exactly-one-of [:from :from-struct]

  :optional-props
  {:from {:types [:string]
          :help "Copy the given file into the Cloudinit image"}
   :from-struct {:types [:struct]
                 :help "Generate a Cloudinit file from the given struct or
                        table"}}

  :mandatory-props
  {:name {:types [:string]
          :help "cloudinit file name. Derived from helper name"}})

(make-helper-fn "cloudinit")
