(use ../lib)

(defhelper :smf dependency
  "Defines a dependency of an SMF service, inside an smf resource."
  :name "Any convenient name - not used internally"

  :mandatory-props
  {:name {:types [:string]
          :help "Convenient name for dependency, derived from resource name"}
   :fmri {:types [:string]
          :help "Dependency FMRI"}
   :restart-on {:types [:string]
                :help "Policy for restarting the parent service if this
                       dependency restarts"
                :default "none"}
   :grouping {:types [:string]
              :help "Which dependencies are required by this service"
              :default "require_all"}
   :type {:types [:string]
          :help "Type of dependency"
          :default "service"}}

  :notes
  ["`network/physical` and `filesystem/local` are hard-coded dependencies."])

(make-helper-fn "dependency")
