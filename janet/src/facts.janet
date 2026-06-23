(use ./lib)
(use ./dsl)

(defn new-fact-cache
  "Returns an empty *collector*. In a function as most tests use it"
  [] @{:ensure @{} :remove @{}})

# Yes, another GLOBAL VARIABLE! Used to cache lazily-evaluated facts.
(def *fact-cache* (new-fact-cache))

(defn- static-fact
  "Returns the value of the given static fact, as a keyword, or nil if the
  fact does not exist."
  [fact-name]
  (let [fact-file (pathcat "/etc/gurp" (string/format "%s.fact" fact-name))]
    (if (os/stat fact-file)
      (-> (slurp fact-file) (string/trim) (keyword))
      nil)))

(defn- fields->tuple
  "Takes an array of fields and returns a tuple in which the first field is a
  downcased keyword of @fields, and second is a number or string value of the
  final value of @fields"
  [fields]
  [(-> fields (first) (->key))
   (parsed-value (last fields))])

(defn- run-wrapper
  "forces fetch-and-cache to use the run-safe-cmd cfunc rather than the stub"
  [cmd]
  ((((fiber/getenv (fiber/root)) 'run-safe-cmd) :value) cmd))

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

(defn- zonename-fact []
  (run-wrapper "/bin/zonename"))

(defn- zones-fact []
  (-> (run-wrapper "/usr/sbin/zoneadm list -cv") (tabular-output->struct 1)))

(defn- type-of-native-zone
  "Tries to work out what kind of zone it is in. Should only be called
  on the assumption that this is a native zone. So far as I can tell, it's
  impossible to tell the difference between ipkg and lipkg, so they both return
  :native"
  []
  (if
    (os/stat "/opt/local/bin/pkgin")
    :pkgsrc
    (do
      (def mount-output (run-wrapper "/usr/sbin/mount"))

      (cond
        (string/find "\n/usr on /usr read only" mount-output)
        :sparse
        (peg/match
          '(* "/ on rpool" (any (if-not "/ROOT/illumos" :S)) "/ROOT/illumos")
          mount-output)
        :illumos
        :native))))

(defn zone-brand-fact
  "Returns the zone brand as a string, or nil if there isn't one."
  []
  # When Gurp makes a zone it creates a static fact. If that's there, use it.
  (if-let [brand (static-fact :zone-brand)]
    brand
    (let [zonename (zonename-fact)]
      # and if the fact isn't there, make an educated guess
      (if (= zonename "global")
        :global
        (match (((zones-fact) zonename) :brand)
          "native" (type-of-native-zone)
          "lx" :lx
          _ (nil))))))

# Don't forget to update RUN_SAVE_CMDS in common/src/constants.rs
(defn fetch-and-cache
  [name]
  (def value
    (match (keyword name)
      :hostname (run-wrapper "/bin/uname -n")
      :zonename (zonename-fact)
      :zone-brand (zone-brand-fact)
      :uname (-> (run-wrapper "/bin/uname -X") (uname-x->struct))
      :zones (zones-fact)
      :links (-> (run-wrapper "/usr/sbin/dladm show-link") (tabular-output->struct))
      :physical-links (-> (run-wrapper "/usr/sbin/dladm show-phys") (tabular-output->struct))
      :ip-interfaces (-> (run-wrapper "/usr/sbin/ipadm show-if") ip-no-loopback)
      :ip-addresses (-> (run-wrapper "/usr/sbin/ipadm show-addr") ip-no-loopback)
      _ (errorf "unknown fact: %s" name)))
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
