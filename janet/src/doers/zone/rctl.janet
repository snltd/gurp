(use ../lib)

(def description-rctl "Define a resource control when creating a zone.")
(def name-is-rctl "RCTL name")
(def mandatory-props-rctl
  {:priv {:types [:string]
          :help "rctl privilege"}
   :action {:types [:string]
            :help "rctl action"}
   :name {:types [:string]
          :help "private field managed by Gurp"}
   :limit {:types [:number]
           :help "rctl limit value"}})
(def optional-props-rctl {})
(def defaults-rctl
  {:priv "privileged"
   :action "deny"})

(defn rctl
  "Given a spec, return a zone rctl struct."
  [name & spec]
  (def spec-struct (make-spec-struct :name name ;spec))
  (def spec-table (checked-spec (spec-with-defaults defaults-rctl spec-struct)
                                mandatory-props-rctl
                                optional-props-rctl))
  (struct :rctl spec-table))
