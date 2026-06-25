(use ./lib)
(import ../collector)

(defdoer :route
  "Manage routes. Note that default routes for zones should be handled by the
  zone's :defrouter property."

  :name-is "The route destination, e.g. '10.10.0.0/16'. For a default route
            specify 'default'."

  :optional-props-ensure
  {:flags {:types [:struct]
           :help "Key-value pairs for flags. If the flag does not take a value,
                  use true"}
   :force-gateway {:types [:boolean]
                   :help "If true, put '-gateway' before the gateway to remove
                         ambiguity"}
   :gateway {:types [:string]
             :help "Gateway for given route."}
   :interface {:types [:string]
               :help "Interface for given route. Conflicts with :gateway"}
   :type {:types [:string]
          :help "Type of route: e.g. 'blackhole', 'reject'"}}

  :mandatory-props-remove
  {:gateway {:types [:string]
             :help "Gateway for given route."}}

  :optional-props-remove
  {:type {:types [:string]
          :help "Type of route: e.g. 'blackhole', 'reject'"}}

  :defaults-ensure
  {:force-gateway false}

  :notes
  ["The `route` command is messy legacy, and it takes all manner of commands.
    This is a best-guess attempt to provide something useful."
   "We only add persistent routes."
   "We only support IPv4."
   "If you created a route of a specific type (e.g. blackhole) be sure to also
    specify the type if you remove it. Otherwise the OS route command can get
    in a tangle."
   "Flags only get set when a route is created. We can't change them on an
    existing route."])

(defn ensure
  "Given a route destination and spec, put an ensure struct in the collector"
  [name & spec]
  (if (has-exactly-one-of? [:gateway :interface] spec)
    (collector/push :ensure doer (make-ensure-resource))
    (error "Provide exactly one of :gateway and :interface")))

(defremove "route")
