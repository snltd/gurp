(use ../lib)

(defhelper :zone :fs
  "Define a filesystem mapping when creating a zone."
  :name-is "The mountpoint inside the zone"

  :mandatory-props
  {:name {:types [:string]
          :help "Mountpoint in zone. This is the name of the resource, and is
               not specified with a key"}
   :special {:types [:string]
             :help "The directory in the global zone"}
   :type {:types [:string]
          :help "The type of fs mount"
          :default "lofs"}}

  :optional-props
  {:options {:types [:tuple]
             :help "Options with which to mount fs inside zone"}})


(make-helper-fn "fs")
