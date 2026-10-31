(use ./definers)
(import ./smf/dependency :prefix "" :export true)
(import ./smf/dependent :prefix "" :export true)
(import ./smf/method :prefix "" :export true)
(import ../collector)

(defdoer :smf
  "Create and install a manifest for an SMF service."
  :name-is "Short name of service. Not used internally"
  (ensure
    (property "fmri" [:string] :mandatory "Service FMRI")
    (property "single-instance" [:boolean] :mandatory
              "Is this a single-instance service?"
              :default true)
    (property "stop-method" [:struct] :mandatory "See 'smf-method'"
              :default {:exec ":kill" :timeout 10})
    (property "default-enabled" [:boolean] :mandatory
              "Start the service when the manifest installs"
              :default true)
    (property "dependency" [:array] :optional "See 'smf-dependency'")
    (property "dependent" [:array] :optional "See 'smf-dependent'")
    (property "description" [:string] :optional "What the service does")
    (property "duration" [:string] :optional
              "Use this to specify 'transient' or 'wait' services")
    (property "properties" [:struct] :optional
              "Create/set properties. (`:keyword :string|:boolean|:number`)")
    (property "property-groups" [:struct] :optional
              "Create property groups. Key is the name, value is the type")
    (property "refresh-method" [] :optional "See 'smf-method'")
    (property "start-method" [:struct] :optional "See 'smf-method'")
    (property "restart-method" [:struct] :optional "See 'smf-method'")
    (property "refresh-method" [:struct] :optional "See 'smf-method'"))

  (notes
    "Generated manifests are written to `/opt/site/lib/smf/manifest`, and
     the output of subsequent runs is compared to the existing reference. If
     the file has changed, the service is re-created. We do it this way because
     it is not possible to compare a running service with a generated manifest."
    "smf/remove stops the service and deletes it from the SMF registry."))

(defn ensure
  "Given a name and a manifest description, return an SMF service ensure struct"
  [name & spec]
  (var modified-spec spec)
  (expand-resource :dependency)
  (expand-resource :dependent)
  (expand-resource :start-method :as-struct true)
  (expand-resource :stop-method :as-struct true)
  (expand-resource :restart-method :as-struct true)
  (expand-resource :refresh-method :as-struct true)

  (let [modified-spec-struct (make-spec-struct ;modified-spec)
        expanded-spec (spec-with-defaults
                        (extract-defaults mandatory-props-ensure)
                        modified-spec-struct)
        spec-table (pinpoint-error
                     :ensure
                     (checked-spec
                       expanded-spec
                       mandatory-props-ensure
                       optional-props-ensure))]

    # Properties must be expanded
    # 
    (if-let [properties (get spec-table :properties)]
      (set (spec-table :properties)
           (tabseq [[prop-name prop-val] :pairs properties]
             prop-name (expand-svc-property prop-val))))

    (collector/push :ensure doer (spec->resource doer name spec-table))))

(make-fn :remove :smf)
