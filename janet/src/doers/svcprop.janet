(use ./lib)
(import ../collector)

(def doer :svcprop)
(def description "Set and remove properties and property groups of an existing
                  SMF service.")
(def name-is "Any valid FMRI of the service whose properties you wish to set")
(def mandatory-props-ensure
  {:properties
   {:types [:struct]
    :help "Properties to create. (:keyword :string|:boolean|:number)"}})
(def optional-props-ensure
  {:property-groups
   {:types [:struct]
    :help "Property groups to create. Key is name, value is type"}})
(def mandatory-props-remove
  {:properties
   {:types [:tuple]
    :help "Properties to remove"}})
(def optional-props-remove
  {:property-groups
   {:types [:struct]
    :help "Property groups to remove"}})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a service property spec, put an ensure struct in the collector"
  [name & spec]
  (def spec-struct
    (pinpoint-error
      :ensure
      (checked-spec (make-spec-struct ;spec)
                    mandatory-props-ensure
                    optional-props-ensure)))
  (def spec-table (spec-with-defaults defaults-ensure spec-struct))

  # Properties must be expanded
  # 
  (if-let [properties (get spec-table :properties)]
    (set (spec-table :properties)
         (tabseq [[prop-name prop-val] :pairs properties]
           prop-name (expand-svc-property prop-val))))

  (collector/push :ensure doer (spec->resource doer name spec-table)))

(defn remove
  "Given a service property spec, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["If not specified, Gurp will infer the types of property values."
   "You can't change the type of an existing property group."])
