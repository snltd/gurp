(use ./definers)
(import ../collector)

(defdoer :svc
  "Manage the state of an existing SMF service."
  :name-is "Service FMRI"
  (ensure
    (property "state" [:string] :mandatory
              "Desired state of service, e.g. `online`")
    (property "reloaded-by" [:tuple] :optional
              "Labels of resources whose alteration triggers service reload"
              :transform-with (partial map string))
    (property "restarted-by" [:tuple] :optional
              "Labels of resources whose alteration triggers service restart"
              :transform-with (partial map string)))

  (notes
    "Because Gurp ends up shelling out to `svcs` and `svcadm`, the service
     name can be any valid FMRI"))

(make-fn :ensure :svc)
