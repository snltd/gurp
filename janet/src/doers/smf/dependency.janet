(use ../lib/definers)

(defhelper :smf dependency
  "Defines a dependency of an SMF service, inside an smf resource."
  :name-is "Any convenient name - not used internally"
  (helper
    (property "name" [:string] :mandatory
              "Convenient name for dependency, derived from resource name")
    (property "fmri" [:string] :mandatory "Dependency FMRI")
    (property "restart-on" [:string] :mandatory
              "Policy for restarting the parent service if this dependency
               restarts"
              :default "none")
    (property "grouping" [:string] :mandatory
              "Which dependencies are required by this service"
              :default "require_all")
    (property "type" [:string] :mandatory "Type of dependency"
              :default "service"))

  (notes
    "`network/physical` and `filesystem/local` are hard-coded dependencies."))

(make-helper-fn :dependency)
