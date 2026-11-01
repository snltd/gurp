(use ./lib/definers)
(import ./lib/lib :only [expand-svc-property])
(import ../collector)

(def allowed-actions ["restart" "refresh"])

(defdoer :svcprop
  "Set and remove properties and property groups of an existing SMF service."
  :name-is "Any valid FMRI of the service whose properties you wish to set"
  (ensure
    (property "properties" [:struct] :mandatory
              "Properties to create. (`:keyword` `:string|:boolean|:number`)"
              :transform-with lib/expand-svc-property)
    (property "property-groups" [:struct] :optional
              "Property groups to create. Key is name, value is type")
    (property "on-change" [:string] :optional
              "Take this action when a value is changed"
              :values allowed-actions))
  (remove
    (property "properties" [:tuple] :mandatory "Properties to remove")
    (property "property-groups" [:tuple] :optional "Property groups to remove"))

  (notes
    "If you want to change a property value on a service instance, you may also
    have to define the property group to which it belongs, as it may not be
    inherited from the base service."
    "When a service restarts on-change, it also refreshes."
    "If not specified, Gurp will infer the types of property values."
    "You can't change the type of an existing property group."))

(make-fn :ensure :svcprop)
(make-fn :remove :svcprop)
