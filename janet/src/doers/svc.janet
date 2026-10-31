(use ./definers)
(import ../collector)

(defdoer :svc
  "Manage the state of an existing SMF service."
  :name-is "Service FMRI"
  (ensure
    (property "state" [:string] :mandatory
              "Desired state of service, e.g. `online`")
    (property "reloaded-by" [:array] :optional
              "Labels of resources whose alteration triggers service reload")
    (property "restarted-by" [:array] :optional
              "Labels of resources whose alteration triggers service restart"))

  (notes
    "Because Gurp ends up shelling out to `svcs` and `svcadm`, the service
    name can be any valid FMRI"))

(defn ensure
  "Given a name and state, put an ensure struct in the collector"
  [name & spec]
  (pinpoint-error
    :ensure
    (let [spec-table (struct/to-table (make-spec-struct ;spec))]
      (when-let [restarters (spec-table :restarted-by)]
        (set (spec-table :restarted-by) (map string restarters)))

      (when-let [reloaders (spec-table :reloaded-by)]
        (set (spec-table :reloaded-by) (map string reloaders)))

      (let [safe-specs
            (checked-spec spec-table
                          mandatory-props-ensure
                          optional-props-ensure)]

        (collector/push :ensure doer (spec->resource doer name safe-specs))))))
