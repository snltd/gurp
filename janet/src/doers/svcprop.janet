(use ./lib)
(import ../collector)

(def allowed-actions ["restart" "refresh"])

(defdoer :svcprop
  "Set and remove properties and property groups of an existing SMF service."
  :name-is "Any valid FMRI of the service whose properties you wish to set"

  :mandatory-props-ensure
  {:properties {:types [:struct]
                :help "Properties to create. (:keyword :string|:boolean|:number)"}}

  :optional-props-ensure
  {:property-groups {:types [:struct]
                     :help "Property groups to create. Key is name, value is type"}
   :on-change {:types [:string]
               :help (string "Take this action when a value is changed. One of "
                             (string/join allowed-actions ", "))}}

  :mandatory-props-remove
  {:properties {:types [:tuple :array]
                :help "Properties to remove"}}

  :optional-props-remove
  {:property-groups {:types [:tuple :array]
                     :help "Property groups to remove"}}

  :notes
  ["If you want to change a property value on a service instance, you may also
    have to define the property group to which it belongs, as it may not be
    inherited from the base service."
   "When a service restarts on-change, it also refreshes."
   "If not specified, Gurp will infer the types of property values."
   "You can't change the type of an existing property group."])

(defn ensure
  "Given a spec, put an ensure struct in the collector"
  [name & spec]

  (let [user-struct (make-spec-struct ;spec)
        spec-struct (pinpoint-error
                      :ensure
                      (checked-spec user-struct
                                    mandatory-props-ensure
                                    optional-props-ensure))
        spec-table (spec-with-defaults defaults-ensure spec-struct)]

    (if-let [action (get spec-table :on-change)]
      (pinpoint-error
        :ensure
        (if-not (has-value? allowed-actions action)
          (errorf "on-change action must be one of %s [got '%s']"
                  (string/join allowed-actions ", ")
                  action))))

    # Properties must be expanded
    (set (spec-table :properties)
         (tabseq [[prop-name prop-val] :pairs (get spec-table :properties ())]
           prop-name (expand-svc-property prop-val)))

    (collector/push :ensure doer (spec->resource doer name spec-table))))

(defremove "svcprop")
