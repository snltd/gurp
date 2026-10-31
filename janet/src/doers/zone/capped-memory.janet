(use ../lib)

(defhelper :zone :capped-memory
  "Set memory caps on a zone being created by the zone doer."
  :at-least-one-of [:physical :swap :locked]

  :optional-props
  {:physical {:types [:string]
              :help "Physical memory cap, with suffix. e.g.
                                      '2G'"}
   :swap {:types [:string]
          :help "Swap cap, with suffix. e.g. '4G'"}
   :locked {:types [:string]
            :help "Locked memory cap, with suffix. e.g. '800M'"}})

(make-helper-fn capped-memory :without-name true)
