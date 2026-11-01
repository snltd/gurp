(import ../../collector)
(use ../lib/definers)

(defhelper :zone :capped-memory
  "Set memory caps on a zone being created by the zone doer."
  (helper
    :at-least-one-of [:physical :swap :locked]
    (property "physical" [:string] :optional
              "Physical memory cap, with suffix. e.g. '2G'")
    (property "swap" [:string] :optional
              "Swap cap, with suffix. e.g. '4G'")
    (property "locked" [:string] :optional
              "Locked memory cap, with suffix. e.g. '800M'")))

(make-helper-fn :capped-memory :no-name true)
