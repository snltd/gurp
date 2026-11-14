(use ./lib/definers)
(import ../collector)

(defdoer :aggr
  "Manage link aggregations"
  :name-is "aggr name"
  (ensure
    (property "links" [:tuple] :mandatory "Underlying links")
    (property "policy" [:string] :mandatory "Port selection policy"
              :values ["L2" "L3" "L4"]
              :default "L4")
    (property "lacp-mode" [:string] :optional "LACP mode"
              :values ["off" "active" "passive"])
    (property "lacp-timer" [:string] :optional "LACP timer value"
              :values ["short" "long"])
    (property "address" [:string] :optional "Unicast hardware address")))

(make-fn :ensure "aggr")
(make-fn :remove "aggr")
