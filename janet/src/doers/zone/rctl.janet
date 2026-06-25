(use ../lib)

(defhelper :zone :rctl
  "Define a resource control when creating a zone."
  :name-is "RCTL name"

  :mandatory-props
  {:priv {:types [:string]
          :help "rctl privilege"}
   :action {:types [:string]
            :help "rctl action"}
   :name {:types [:string]
          :help "private field managed by Gurp"}
   :limit {:types [:number]
           :help "rctl limit value"}}

  :defaults
  {:priv "privileged"
   :action "deny"})

(defn rctl
  "Given a spec, return a zone rctl struct."
  [name & spec]
  (let [spec-struct (make-spec-struct :name name ;spec)
        expanded-spec (spec-with-defaults defaults-rctl spec-struct)
        spec-table (pinpoint-error :rctl
                                   (checked-spec expanded-spec
                                                 mandatory-props-rctl
                                                 optional-props-rctl))]

    (struct :rctl spec-table)))
