(use ../definers)

(defhelper :smf :dependent
  "Defines a dependent of an SMF service, inside an smf resource."
  :name "Any convenient name - not used internally"

  :mandatory-props
  {:name {:types [:string]
          :help "Convenient name for dependent, derived from resource name"}
   :fmri {:types [:string]
          :help "Dependent FMRI"}
   :restart-on {:types [:string]
                :help "Policy for restarting parent service if this dependent
                       restarts"
                :default "none"}
   :grouping {:types [:string]
              :help "Which dependents are required by this service"
              :default "require_all"}
   :type {:types [:string]
          :help "Type of dependent"
          :default "service"}})

(make-helper-fn dependent)
