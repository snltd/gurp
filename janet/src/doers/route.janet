(use ./lib)
(import ../collector)

(def doer :route)
(def description "Manage routes. Note that default routes for zones should be
                 handled by the zone's :defrouter property.")
(def name-is "The route destination, e.g. 10.10.0.0/16")
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:flags {:types [:struct]
           :help "Key-value pairs for flags. If the flag does not take a value,
                  use true"}
   :force-gateway {:types [:boolean]
                   :help "If true, put '-gateway' before the gateway to remove
                         ambiguity"}
   :gateway {:types [:string]
             :help "Gateway for given route. For a default route specify
                   'default'"}
   :interface {:types [:string]
               :help "Interface for given route. Conflicts with :gateway"}
   :type {:types [:string]
          :help "Type of route: e.g. 'blackhole', 'reject'"}})
(def mandatory-props-remove
  {:gateway {:types [:string]
             :help "Gateway for given route. For a default route specify
                   'default'"}})
(def optional-props-remove {})
(def defaults-ensure
  {:force-gateway false})
(def defaults-remove {})

(defn ensure
  "Given a route destination and spec, put an ensure struct in the collector"
  [name & spec]

  (if-not (has-exactly-one-of? [:gateway :interface] spec)
    (error "Provide exactly one of :gateway and :interface"))

  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a route and a gateway, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["The `route` command is messy legacy, and it takes all manner of commands.
    This is a best-guess attempt to provide something useful."
   "We only add persistent routes."
   "We only support IPv4."
   "Flags only get set when a route is created. We can't change them on an
    existing route."])
