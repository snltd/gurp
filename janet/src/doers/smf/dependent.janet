(use ../lib)

(def description-dependent "Defines a dependent of an SMF service, inside an
                            smf resource.")
(def name-dependent "Any convenient name - not used internally")
(def optional-props-dependent
  {:restart-on {:types [:string]
                :help "Policy for restarting this service if dependent restarts"}
   :grouping {:types [:string]
              :help "Which dependencies are required by this service"}
   :type {:types [:string]
          :help "Type of dependent"}})
(def mandatory-props-dependent
  {:name {:types [:string]
          :help "Convenient name for dependent, derived from resource name"}
   :fmri {:types [:string]
          :help "Dependent FMRI"}})

(def defaults-dependent
  {:restart-on "none"
   :grouping "require_all"
   :type "service"})

(defn dependent
  "A convenience function to help produce an SMF dependent"
  [name & spec]
  (def spec-struct
    (checked-spec
      (make-spec-struct :name name ;spec)
      mandatory-props-dependent optional-props-dependent))

  (def all-specs (spec-with-defaults defaults-dependent spec-struct))
  (struct :dependencies all-specs))
