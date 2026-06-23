(use ./lib)
(import ./smf/dependency :prefix "" :export true)
(import ./smf/dependent :prefix "" :export true)
(import ./smf/method :prefix "" :export true)
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

  (let [modified-spec-struct (make-spec-struct ;modified-spec)
        spec-struct (pinpoint-error
                      :ensure
                      (checked-spec
                        modified-spec-struct
                        mandatory-props-ensure
                        optional-props-ensure))

        spec-table (spec-with-defaults defaults-ensure spec-struct)]

    # Properties must be expanded
    # 
    (if-let [properties (get spec-table :properties)]
      (set (spec-table :properties)
           (tabseq [[prop-name prop-val] :pairs properties]
             prop-name (expand-svc-property prop-val))))

    (collector/push :ensure doer (spec->resource doer name spec-table))))

(defn remove
  "Given an apk package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["Generated manifests are written to `/opt/site/lib/smf/manifest`, and
   the output of subsequent runs is compared to the existing reference. If the
   file has changed, the service is re-created. We do it this way because it is
   not possible to compare a running service with a generated manifest."
   "smf/remove stops the service and deletes it from the SMF registry."])
