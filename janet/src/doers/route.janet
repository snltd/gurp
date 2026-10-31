(use ./definers)
(import ../collector)

(defdoer :route
  "Manage routes. Note that default routes for zones should be handled by the
  zone's :defrouter property."
  :name-is "The route destination, e.g. '10.10.0.0/16'. For a default route
            specify 'default'."
  (ensure
    :exactly-one-of [:gateway :interface]
    (property "flags" [:struct] :optional
              "Key-value pairs for flags. If the flag does not take a value,
               use `true`")
    (property "force-gateway" [:boolean] :optional
              "If `true`, put `-gateway` before the gateway to remove ambiguity")
    (property "gateway" [:string] :optional "Gateway for given route")
    (property "interface" [:string] :optional
              "Interface for given route. Conflicts with `:gateway`")
    (property "type" [:string] :optional
              "Type of route: e.g. `blackhole`, `reject`"))

  (remove
    (property "gateway" [:string] :mandatory "Gateway for given route.")
    (property "type" [:string] :optional
              "Type of route: e.g. `blackhole`, `reject`"))

  (notes
    "The `route` command is messy legacy, and it takes all manner of commands.
     This is a best-guess attempt to provide something useful."
    "We only add persistent routes."
    "We only support IPv4."
    "If you created a route of a specific type (e.g. blackhole) be sure to also
     specify the type if you remove it. Otherwise the OS route command can get
     in a tangle."
    "Flags only get set when a route is created. We can't change them on an
     existing route."))

(make-fn :ensure :route)
(make-fn :remove :route)
