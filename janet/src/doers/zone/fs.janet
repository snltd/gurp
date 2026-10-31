(import ../../collector)
(use ../definers)

(defhelper :zone :fs
  "Define a filesystem mapping when creating a zone."
  :name-is "The mountpoint inside the zone"
  (helper
    (property "name" [:string] :mandatory
              "Mountpoint in zone. This is the name of the resource, and is
               not specified with a key")
    (property "special" [:string] :mandatory "The directory in the global zone")
    (property "type" [:string] :mandatory "The type of fs mount"
              :default "lofs")
    (property "options" [:tuple] :optional
              "Options with which to mount fs inside zone")))

(make-helper-fn "fs")
