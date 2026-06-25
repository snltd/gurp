(use ../lib)

(defhelper :zone :fs
  "Define a filesystem mapping when creating a zone."
  :name-is "The mountpoint inside the zone"

  :optional-props
  {:type {:types [:string]
          :help "The type of fs mount"}
   :options {:types [:tuple]
             :help "Options with which to mount fs inside zone"}}

  :mandatory-props
  {:dir {:types [:string]
         :help "Mountpoint in zone. This is the name of the resource, and is
               not specified with a key"}
   :special {:types [:string]
             :help "The directory in the global zone"}}

  :defaults {:type "lofs"})

(defn fs
  "Given a spec, return a zone fs struct."
  [name & spec]
  (let [spec-struct (make-spec-struct :dir name ;spec)
        expanded-spec (spec-with-defaults defaults-fs spec-struct)
        spec-table (pinpoint-error :fs
                                   (checked-spec expanded-spec
                                                 mandatory-props-fs
                                                 optional-props-fs))]

    (struct :fs spec-table)))
