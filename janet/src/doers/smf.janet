(use ./lib/definers)
(import ../collector)
(import ./lib/lib :only [expand-svc-property])
(import ./smf/dependency :prefix "" :export true)
(import ./smf/dependent :prefix "" :export true)
(import ./smf/method :prefix "" :export true)

(defdoer :smf
  "Create and install a manifest for an SMF service."
  :name-is "Short name of service. Not used internally"
  (ensure
    (property "description" [:string] :optional "What the service does")
    (property "fmri" [:string] :mandatory "Service FMRI")
    (property "single-instance" [:boolean] :mandatory
              "Is this a single-instance service?"
              :default true)
    (property "default-enabled" [:boolean] :mandatory
              "Start the service when the manifest installs"
              :default true)
    (property "dependency" [:tuple] :optional
              "Service dependencies. See 'smf-dependency'")
    (property "dependent" [:tuple] :optional
              "Dependent services. See 'smf-dependent'")
    (property "duration" [:string] :optional
              "Use this to specify 'transient' or 'wait' services")
    (property "properties" [:struct] :optional
              "Create/set properties. (`:keyword :string|:boolean|:number`)"
              :transform-with lib/expand-svc-property)
    (property "property-groups" [:struct] :optional
              "Create property groups. Key is the name, value is the type")
    (property "start-method" [:struct] :optional "See 'smf-method'")
    (property "stop-method" [:struct] :mandatory "See 'smf-method'"
              :default {:exec ":kill" :timeout 10})
    (property "restart-method" [:struct] :optional "See 'smf-method'")
    (property "refresh-method" [:struct] :optional "See 'smf-method'"))

  (notes
    "Generated manifests are written to `/opt/site/lib/smf/manifest`, and
     the output of subsequent runs is compared to the existing reference. If
     the file has changed, the service is re-created. We do it this way because
     it is not possible to compare a running service with a generated manifest."
    "smf/remove stops the service and deletes it from the SMF registry."))

(make-fn :ensure :smf)
(make-fn :remove :smf)
