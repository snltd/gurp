(use ./lib)
(import ../collector)

(def doer :smf)
(def description "Create and install a manifest for an SMF service.")
(def name-is "Short name of service. Not used internally")
(def mandatory-props-ensure
  {:fmri {:types [:string]
          :help "Service FMRI"}})
(def optional-props-ensure
  {:dependencies
   {:types [:array]
    :help "See 'smf-dependency'"}
   :dependents
   {:types [:array]
    :help "See 'smf-dependent'"}
   :description
   {:types [:string]
    :help "What the service does"}
   :duration
   {:types [:string]
    :help "Use this to specify 'transient' or 'wait' services"}
   :properties
   {:types [:struct :table]
    :help "Create/set properties.(:keyword :string|:boolean|:number)"}
   :property-groups
   {:types [:struct :table]
    :help "Create property groups. Key is the name, value is the type"}
   :refresh-method
   {:types []
    :help "See 'smf-method'"}
   :single-instance
   {:types [:boolean]
    :help "Is this a single-instance service"}
   :start-method
   {:types [:struct :table]
    :help "See 'smf-method'"}
   :stop-method
   {:types [:struct :table]
    :help "See 'smf-method'"}
   :restart-method
   {:types [:struct :table]
    :help "See 'smf-method'"}
   :refresh-method
   {:types [:struct :table]
    :help "See 'smf-method'"}
   :default-enabled
   {:types [:boolean]
    :help "Start the service when the manifest installs"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:single-instance true
   :stop-method {:exec ":kill" :timeout 10}
   :default-enabled true})
(def defaults-remove {})

(defn- expand-svc-property
  "Turns a svcprop value into a struct describing a typed value"
  [value]
  (match (type value)
    :keyword value
    :number {:type "integer" :value value}
    :boolean {:type "boolean" :value value}
    _ {:type "astring" :value value}))

(defn ensure
  "Given a name and a manifest description, return an SMF service ensure struct"
  [name & spec]
  (var modified-spec spec)
  (expand-resource :dependencies)
  (expand-resource :dependents)
  (expand-resource :start-method :as-struct true)
  (expand-resource :stop-method :as-struct true)
  (expand-resource :restart-method :as-struct true)
  (expand-resource :refresh-method :as-struct true)

  (def modified-spec-struct (make-spec-struct ;modified-spec))
  (def spec-struct (checked-spec modified-spec-struct mandatory-props-ensure optional-props-ensure))
  (def spec-table (spec-with-defaults defaults-ensure spec-struct))

  # Properties must be expanded
  # 
  (if-let [properties (get spec-table :properties)]
    (set (spec-table :properties)
         (tabseq [[prop-name prop-val] :pairs properties]
           prop-name (expand-svc-property prop-val))))

  (collector/push :ensure doer (spec->resource doer name spec-table)))

(defn remove
  "Given an apk package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def allowed-methods ["start" "stop" "refresh" "reload"])
(def optional-props-method
  {:user {:types [:string]
          :help "User the method runs as"}
   :group {:types [:string]
           :help "Group the method runs as"}
   :privileges {:types [:tuple]
                :help "Privileges the method has. Use ! to remove them"}
   :environment {:types [:struct :table]
                 :help "Environment variables set inside context"}})
(def mandatory-props-method
  {:exec
   {:types [:string]
    :help "Method or command to execute"}
   :timeout
   {:types [:number]
    :help "Seconds until method times out"}})

(def defaults-method {:timeout 60})
(def context-props [:user :group :privileges :environment])

(defn method
  "Produce an SMF exec_method, with a context"
  [name & spec]
  (if-not (has-value? allowed-methods name)
    (error
      (string "smf/method name must be one of " (comma-sep allowed-methods))))

  (def spec-table (spec-with-defaults defaults-method (make-spec-struct ;spec)))

  # We have to move context related properties (context-props) into a
  # :context struct

  (var context-table @{})

  (loop [prop :in context-props]
    (when-let [spec-value (get spec-table prop)]
    (def value-to-move (if (= prop :privileges)
                         (string/join spec-value ",")
                         spec-value))

    (set (context-table prop) value-to-move)
    (set (spec-table prop) nil)))

  (if-not (empty? context-table)
    (set (spec-table :context) (table/to-struct context-table)))

  (struct (keyword (string name "-method")) spec-table))


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
          :help "Convenient name for dependency, derived from resource name"}
   :fmri {:types [:string]
          :help "Dependent FMRI"}})

(def defaults-dependent
  {:restart-on "none"
   :grouping "require_all"
   :type "service"})

(defn dependent
  "A convenience function to help produce an SMF dependent"
  [name & spec]
  (def spec-struct (checked-spec (make-spec-struct :name name ;spec) mandatory-props-dependency optional-props-dependency))
  (def all-specs (spec-with-defaults defaults-dependent spec-struct))
  (struct :dependencies all-specs))
