(use ./lib)
(import ../collector)

(def doer :svc)
(def description "Manage the state of an existing SMF service.")
(def name-is "Service FMRI")
(def mandatory-props-ensure
  {:state
   {:types [:string]
    :help "Desired state of service, e.g. 'online'"}})
(def optional-props-ensure
  {:reloaded-by
   {:types [:array]
    :help "Labels of resources whose alteration triggers service reload"}
   :restarted-by
   {:types [:array]
    :help "Labels of resources whose alteration triggers service restart"}})
(def defaults-ensure
  { :restarted-by []
   :reloaded-by []})

(defn ensure
  "Given a name and state, put an ensure struct in the collector"
  [name & spec]

  (def spec-struct (make-spec-struct ;spec))
  (def spec-table (spec-with-defaults defaults-ensure spec-struct))

  (if-let [restarters (spec-table :restarted-by)]
    (set (spec-table :restarted-by) (map string restarters)))

  (if-let [reloaders (spec-table :reloaded-by)]
    (set (spec-table :reloaded-by) (map string reloaders)))

  (def safe-specs (checked-spec spec-table
                                mandatory-props-ensure
                                optional-props-ensure))

  (collector/push :ensure doer (spec->resource doer name safe-specs)))

(def notes
  ["Because Gurp ends up shelling out to `svcs` and `svcadm`, the service
    name can be any valid FMRI"])
