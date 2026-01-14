(use ../lib)

(def description-dependency "Defines a dependency of an SMF service, inside an
                            smf resource.")
(def name-dependency "Any convenient name - not used internally")
(def optional-props-dependency
  {:restart-on {:types [:string]
                :help "Policy for restarting this service if dependency restarts"}
   :grouping {:types [:string]
              :help "Which dependencies are required by this service"}
   :type {:types [:string]
          :help "Type of dependency"}})
(def mandatory-props-dependency
  {:name {:types [:string]
          :help "Convenient name for dependency, derived from resource name"}
   :fmri {:types [:string]
          :help "Dependency FMRI"}})

(def defaults-dependency
  {:restart-on "none"
   :grouping "require_all"
   :type "service"})

(defn dependency
  "A convenience function to help produce an SMF dependency"
  [name & spec]
  (def spec-struct (checked-spec (make-spec-struct :name name ;spec) mandatory-props-dependency optional-props-dependency))
  (def all-specs (spec-with-defaults defaults-dependency spec-struct))
  (struct :dependencies all-specs))
