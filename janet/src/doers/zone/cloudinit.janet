(import ../../collector)
(use ../lib/definers)

(defhelper :zone :cloudinit
  "Describe cloudinit config inside a zone resource"
  :name-is "cloudinit file name"
  (helper
    :exactly-one-of [:from :from-struct]
    (property "name" [:string] :mandatory
              "cloudinit file name. Derived from helper name")
    (property "from" [:string] :optional
              "Copy the given file into the Cloudinit image")
    (property "from-struct" [:struct] :optional
              "Generate a Cloudinit file from the given struct or table")))

(make-helper-fn :cloudinit)
