(use ./dsl)
(use ./lib)

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

(defn fetch-and-cache
  [name]
  (def value
    (match (keyword name)
      :hostname (run-cmd "/bin/uname -n")
      :zonename (run-cmd "/bin/zonename")
      :uname (-> "bin/uname -X" (run-cmd) (uname-x->struct))
      :zones (-> "/usr/sbin/zoneadm list -cv" (run-cmd) (tabular-output->struct 1))
      :links (-> "/usr/sbin/dladm show-link" (run-cmd) (tabular-output->struct))
      :ip-interfaces (-> "/usr/sbin/ipadm show-if" (run-cmd) ip-no-loopback)
      :ip-addresses (-> "/usr/sbin/ipadm show-addr" (run-cmd) ip-no-loopback)
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
