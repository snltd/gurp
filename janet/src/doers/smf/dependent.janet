(use ../definers)

(defhelper :smf :dependent
  "Defines a dependent of an SMF service, inside an smf resource."
  :name-is "Any convenient name - not used internally"
  (helper
    (property "name" [:string] :mandatory
              "Convenient name for dependent, derived from resource name")
    (property "fmri" [:string] :mandatory "Dependent FMRI")
    (property "restart-on" [:string] :mandatory
              "Policy for restarting parent service if this dependent restarts"
              :default "none")
    (property "grouping" [:string] :mandatory
              "Which dependents are required by this service"
              :default "require_all")
    (property "type" [:string] :mandatory "Type of dependent"
              :default "service")))

(make-helper-fn :dependent)
