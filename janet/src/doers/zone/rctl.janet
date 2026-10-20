(use ../lib)

(defhelper :zone :rctl
  "Define a resource control when creating a zone."
  :name-is "RCTL name"

  :mandatory-props
  {:priv {:types [:string]
          :help "rctl privilege"
          :default "privileged"}
   :action {:types [:string]
            :help "rctl action"
            :default "deny"}
   :name {:types [:string]
          :help "private field managed by Gurp"}
   :limit {:types [:number]
           :help "rctl limit value"}})

(make-helper-fn rctl)
