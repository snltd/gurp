(use ./lib)
(use ./dsl)

(defn new-fact-cache
  "Returns an empty *collector*. In a function as most tests use it"
  [] @{:ensure @{} :remove @{}})

# Yes, another GLOBAL VARIABLE! Used to cache lazily-evaluated facts.
(def *fact-cache* (new-fact-cache))

(defn- fields->tuple
  "Takes an array of fields and returns a tuple in which the first field is a
  downcased keyword of @fields, and second is a number or string value of the
  final value of @fields"
  [fields]
  [(-> fields (first) (->key)) (parsed-value (last fields))])

(defn uname-x->struct
  "Return a struct representation of the output of uname -X"
  [raw]
  (->> raw
       (lines)
       (map fields)
       (map fields->tuple)
       (tabular-data->struct)))

(defn ip-no-loopback
  [raw]
  (->> raw
       (lines)
       (filter |(not (string/has-prefix? "lo" $)))
       (tabular-rows->struct)))

(defn- run-wrapper
  "forces fetch-and-cache to use the run-safe-cmd cfunc rather than the stub"
  [cmd]
  ((((fiber/getenv (fiber/root)) 'run-safe-cmd) :value) cmd))

(defn fetch-and-cache
  [name]
  (def value
    (match (keyword name)
      :hostname (run-wrapper "/bin/uname -n")
      :zonename (run-wrapper "/bin/zonename")
      :uname (-> (run-wrapper "/bin/uname -X") (uname-x->struct))
      :zones (-> (run-wrapper "/usr/sbin/zoneadm list -cv") (tabular-output->struct 1))
      :links (-> (run-wrapper "/usr/sbin/dladm show-link") (tabular-output->struct))
      :ip-interfaces (-> (run-wrapper "/usr/sbin/ipadm show-if") ip-no-loopback)
      :ip-addresses (-> (run-wrapper "/usr/sbin/ipadm show-addr") ip-no-loopback)
      _ (error (string "unknown fact: " name))))
  (set (*fact-cache* name) value))

(defn fact
  "Return, if it exists, the built-in fact with the given name. Facts are
  evaluated lazily and cached in the *fact-cache* global. If you want to bypass
  the cache, pass in a truthy second argument."
  [name &opt bypass-cache]

  (if-let [_ (not bypass-cache)
           value (get *fact-cache* name)]
    value
    (fetch-and-cache name)))

# Stub functions so the library compiles as a module
(defn- run-safe-cmd [cmd] "stub")
(defn- run-cmd [cmd &opt args] "stub")
