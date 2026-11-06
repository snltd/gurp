(import ./dsl :only [fields lines pathcat tabular-rows->struct
                     tabular-output->struct])
(import ./lib :only [->key parsed-value tabular-data->struct])

# Returns an empty *collector*. In a function as most tests use it
(defn new-fact-cache [] @{:ensure @{} :remove @{}})

# Yes, another GLOBAL VARIABLE! Used to cache lazily-evaluated facts.
(def *fact-cache* (new-fact-cache))

(defn- static-fact
  "Returns the value of the given static fact, as a keyword, or nil if the
  fact does not exist."
  [fact-name]
  (let [fact-file (dsl/pathcat "/etc/gurp" (string/format "%s.fact" fact-name))]
    (if (os/stat fact-file)
      (-> (slurp fact-file) (string/trim) (keyword))
      nil)))

# Takes an array of fields and returns a tuple in which the first field is a
# downcased keyword of @fields, and second is a number or string value of the
# final value of @fields
(defn- fields->tuple
  [fields]
  [(-> fields (first) (lib/->key)) (lib/parsed-value (last fields))])

# Forces fetch-and-cache to use the run-safe-cmd cfunc rather than the stub
(defn- run-wrapper [cmd]
  ((((fiber/getenv (fiber/root)) 'run-safe-cmd) :value) cmd))

# Return a struct representation of the output of uname -X
(defn uname-x->struct [raw]
  (->> raw (dsl/lines)
       (map dsl/fields)
       (map fields->tuple)
       (lib/tabular-data->struct)))

(defn ip-no-loopback [raw]
  (->> raw
       (dsl/lines)
       (filter |(not (string/has-prefix? "lo" $)))
       (dsl/tabular-rows->struct)))

(defn- zonename-fact []
  (run-wrapper "/bin/zonename"))

(defn- zones-fact []
  (-> (run-wrapper "/usr/sbin/zoneadm list -cv")
      (dsl/tabular-output->struct 1)))

# Tries to work out what kind of zone it is in. Should only be called on the
# assumption that this is a native zone. So far as I can tell, it's impossible
# to tell the difference between ipkg and lipkg, so they both return :native
(defn- type-of-native-zone []
  (if (os/stat "/opt/local/bin/pkgin")
    :pkgsrc
    (let [mount-output (run-wrapper "/usr/sbin/mount")]
      (cond (string/find "\n/usr on /usr read only" mount-output) :sparse
        (peg/match
          '(* "/ on rpool" (any (if-not "/ROOT/illumos" :S)) "/ROOT/illumos")
          mount-output) :illumos
        :native))))

# Returns the zone brand as a keyword, or nil if there isn't one. When Gurp
# makes a zone it creates a static fact. If that's there, use it. If the fact
# isn't there, we make an educated guess.
(defn zone-brand-fact []
  (if-let [brand (static-fact :zone-brand)]
    brand
    (let [zonename (zonename-fact)]
      (if (= zonename "global")
        :global
        (match
          (((zones-fact) zonename) :brand)
          "native" (type-of-native-zone)
          "lx" :lx
          _ (nil))))))

(defmacro deffact [k typ desc fnc]
  (struct k (struct :description desc :type typ :fn fnc)))

(defn fact->fn* []
  (merge-into
    @{}
    (deffact :hostname :string
      "The name of the host on which Gurp is running, taken from `hostname.`"
      (fn [] (run-wrapper "/bin/uname -n")))

    (deffact :ip-addresses :struct
      "A struct with address names as keys, and structs of properties as values.
       Value keys are the headers of `ipadm show-addr`."
      (fn [] (-> (run-wrapper "/usr/sbin/ipadm show-addr") ip-no-loopback)))

    (deffact :ip-interfaces :struct
      "A struct with interface names as keys, and structs of properties as
       values. Value keys are the headers of `ipadm show-if`."
      (fn [] (-> (run-wrapper "/usr/sbin/ipadm show-if") ip-no-loopback)))

    (deffact :links :struct
      "A struct with link names names as keys, and structs of properties as
       values. Value keys are the headers of `dladm show-link`."
      (fn [] (-> (run-wrapper "/usr/sbin/dladm show-link")
                 (dsl/tabular-output->struct))))

    (deffact :physical-links :struct
      "A struct with link names names as keys, and structs of properties as
       values. Value keys are the headers of `dladm show-link`."
      (fn [] (-> (run-wrapper "/usr/sbin/dladm show-phys")
                 (dsl/tabular-output->struct))))

    (deffact :uname :struct "A struct made of the output of `uname -X`."
      (fn [] (-> (run-wrapper "/bin/uname -X") (uname-x->struct))))

    (deffact :zfs-filesystems :struct
      "A struct with ZFS filesystem names names as keys, and structs of
       properties as values. Value keys are the headers of `zfs list`."
      (fn [] (-> (run-wrapper "/usr/sbin/zfs list -p")
                 (dsl/tabular-output->struct))))

    (deffact :zone-brand :string
      "The brand of the zone in which Gurp is running. If Gurp created the zone,
       it will have left a fact from which this value is derived, otherwise a
       best-guess effort is made."
      (fn [] (zone-brand-fact)))

    (deffact :zonename :string
      "The name of the host on which Gurp is running, taken from `hostname`."
      (fn [] (zonename-fact)))

    (deffact :zones :struct
      "A struct with zone names names as keys, and structs of properties as
       values. Value keys are the headers of `zoneadm list -cv`."
      (fn [] (zones-fact)))

    (deffact :zpools :struct
      "A struct with ZFS pool names names as keys, and structs of properties as
       values. Value keys are the headers of `zpool list`."
      (fn [] (-> (run-wrapper "/usr/sbin/zpool list -p")
                 (dsl/tabular-output->struct))))))

(def fact->fn (fact->fn*))

# Don't forget to update RUN_SAVE_CMDS in crates/common/src/constants.rs
(defn fetch-and-cache [name]
  (def value (if-let [fun (fact->fn name)]
               ((fun :fn))
               (errorf "unknown fact: %s" name)))
  (set (*fact-cache* name) value))

# Return, if it exists, the built-in fact with the given name. Facts are
# evaluated lazily and cached in the *fact-cache* global. If you want to bypass
# the cache, pass in a truthy second argument."
(defn fact [name &opt bypass-cache]
  (if-let [_ (not bypass-cache)
           value (get *fact-cache* name)]
    value
    (fetch-and-cache name)))

# Stub functions so the library compiles as a module
(defn- run-safe-cmd [cmd] "stub")
(defn- run-cmd [cmd &opt args] "stub")
