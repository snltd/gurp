(use ./lib)
(import ../collector)

(defdoer :svc
  "Manage the state of an existing SMF service."
  :name-is "Service FMRI"

  :mandatory-props-ensure
  {:state {:types [:string]
           :help "Desired state of service, e.g. 'online'"}}

  :optional-props-ensure
  {:reloaded-by {:types [:array]
                 :help "Labels of resources whose alteration triggers service reload"}
   :restarted-by {:types [:array]
                  :help "Labels of resources whose alteration triggers service restart"}}

  :defaults-ensure {:restarted-by []
                    :reloaded-by []}

  :notes
  ["Because Gurp ends up shelling out to `svcs` and `svcadm`, the service
    name can be any valid FMRI"])

(defn ensure
  "Given a name and state, put an ensure struct in the collector"
  [name & spec]
  (let [spec-struct (make-spec-struct ;spec)
        spec-table (spec-with-defaults defaults-ensure spec-struct)]

    (if-let [restarters (spec-table :restarted-by)]
      (set (spec-table :restarted-by) (map string restarters)))

    (if-let [reloaders (spec-table :reloaded-by)]
      (set (spec-table :reloaded-by) (map string reloaders)))

    (let [safe-specs (pinpoint-error
                       :ensure (checked-spec spec-table
                                             mandatory-props-ensure
                                             optional-props-ensure))]

      (collector/push :ensure doer (spec->resource doer name safe-specs)))))
