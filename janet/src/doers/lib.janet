#
# 
# Functions, macros, and bindings required by the doer modules in this directory.
# 
(import ../dsl :only [qualify-from-path])
(import ../collector)
(import ../defs :only [ip-protocols])


(defn comma-sep
  "Return a comma-separated string of the items in list. Everything is literal,
  so keywords stay keywords, and quoted strings keep their quotes."
  [list]
  (string/join (map |(string/format "%p" $) list) ", "))

# probably goes in validators
(defn keys-in-struct
  "Returns the elements of key-list which are keys in struct-to-check"
  [key-list struct-to-check]
  (filter |(has-value? key-list $) (keys struct-to-check)))

(defn expand-svc-property
  "Turns the keys into a svcprop struct into structs describing a typed value"
  [value-struct]
  (tabseq [[prop-name value] :pairs value-struct]
    prop-name (match (type value)
                :keyword value
                :number {:type "integer" :value value}
                :boolean {:type "boolean" :value value}
                _ {:type "astring" :value value})))


(comment
  (defn protocol-opts
    []
    (tabseq [protocol :in defs/ip-protocols]
      '(property (string protocol) [:struct] :optional
                 (string/format "key-value pairs of valid %s properties" protocol))))

  (defn make-spec-struct
    "Turn a list of property key/value pairs into a struct or, raise a more
  useful error than 'expected even number of arguments'"
    [& spec]
    (try
      (struct ;spec)
      ([e]
        (errorf "unable to create struct from %d arg(s):  %p: %s"
                (length spec)
                spec
                e))))

  (defn resource-id
    "Safely generate a resource ID"
    [resource-type role name]
    (let [name (if (nil? name) "NO-NAME" (string/replace-all "/" "_" name))]
      (string/format "/%s/%s/%s" role resource-type name)))

  (defmacro expand-resource
    "Group the results of helpers (e.g. zone/fs) into a list under a single key.
  Partitions `modified-spec` items whose keys do or do not match
  the given `key`. The matches ('is' group) are flattened into a single array
  (or, if `:as-struct` is passed, reduced to the first matching struct) and
  stored under `key`. Non-matching items ('is-not' group) are preserved."
    [key &keys {:as-struct as-struct}]
    (with-syms [$is-key $key-list $vals]
      ~(do
         (let [$is-key (group-by
                         |(and (struct? $) (deep= @[,key] (keys $)))
                         modified-spec)]

           (when-let [$key-list ($is-key true)]
             (let [$vals (mapcat values $key-list)]
               (set modified-spec
                    (tuple
                      ;(get $is-key false @[])
                      ,key
                      (if ,as-struct (first $vals) $vals)))))))))

  (defn xpander
    [expand-resources spec]
    (if (empty? expand-resources)
      spec
      (do
        (var modified-spec spec)
        (loop [[helper as-struct] :pairs expand-resources]
          ~(print "helper is " helper " as-struct is " as-struct)
          (if as-struct
            (expand-resource helper :as-struct true)
            (expand-resource helper)))

        modified-spec)))

  (defmacro make-ensure-resource
    "Pulls together some boilerplate in doer ensure functions"
    []
    ~(let [spec-struct (make-spec-struct ;(xpander expand-resources spec))
           spec-table (expand-from-value (struct/to-table spec-struct))
           all-specs (spec-with-defaults
                       (extract-defaults mandatory-props-ensure)
                       spec-table)
           safe-specs (checked-spec all-specs
                                    mandatory-props-ensure
                                    optional-props-ensure)]

       (spec->resource doer name safe-specs)))

  (defmacro make-remove-resource
    "Pulls together some boilerplate in doer remove functions"
    []
    ~(pinpoint-error
       :remove
       (let [spec-struct (make-spec-struct ;spec)
             all-specs (spec-with-defaults
                         (extract-defaults mandatory-props-remove)
                         spec-struct)
             safe-specs (checked-spec all-specs
                                      mandatory-props-remove
                                      optional-props-remove)]

         (spec->resource doer name safe-specs))))

  (defn has-none-or-one-of?
    "Checks whether a spec contains none or one of the required-keys"
    [required-keys spec]
    (<= (length (keys-in-struct required-keys (make-spec-struct ;spec))) 1))

  (defn key-has-value?
    "Checks whether the given key has an acceptable value, errors if key is not
   set"
    [spec key acceptable-values]
    (if-let [given-value (get (struct ;spec) key)]
      (check-key-value key given-value acceptable-values)
      (errorf "key-has-value? did not find key %p" key)))

  (defmacro table->flat-tuple
    "One-level-flattens a struct or table, including its keys"
    [table]
    ~(flatten (pairs ,table)))

  (defn group-ip-properties
    "Move IP protocol properties into a separate :protocol property"
    [mandatory-props optional-props & spec]

    (let [spec-struct (make-spec-struct ;spec)
          temp-spec-table (checked-spec spec-struct mandatory-props optional-props)
          parts (->> temp-spec-table
                     (table->flat-tuple)
                     (partition 2)
                     (partition-by |(has-value? ip-protocols (first $)))
                     (map flatten))

          spec-table (table ;(get parts 1 []))]

      (when-let [protocol-parts (get parts 0)
                 protocol-table (struct ;protocol-parts)]
        (set (spec-table :protocols) protocol-table))

      spec-table))

  (defn expand-from-value
    "Used by any resource that takes a :from. e.g. file and system-cert. Give it
  a spec table, and it will turn relative :from paths into fully qualified paths
  or server references, depending on the current execution environment. Returns
  a modified spec-table."
    [spec-table]
    (when-let [from-path (spec-table :from)]
      (if-let [server-name (dyn :server-name)]
        (do
          (set (spec-table :from) nil)
          (set (spec-table :from-url) (server-url server-name from-path))
          (set (spec-table :url-is-server) true))
        (let [url-or-qualified-path
              (if (string/find "://" from-path)
                from-path
                (dsl/qualify-from-path from-path))]
          (set (spec-table :from) url-or-qualified-path))))

    spec-table)

  # A doer requires these properties, but we use a prototype struct so you don't
  # have to explicitly define them if they're empty.
  (def- doer-defaults {:mandatory-props-ensure {}
                       :optional-props-ensure {}
                       :mandatory-props-remove {}
                       :optional-props-remove {}
                       :exactly-one-of []
                       :at-least-one-of []
                       :expand-resources {}})
  (defmacro defdoer
    "Creates the defs required for a doer"
    [doer-name description & spec]
    (def spec-struct (->> ;spec
                          (struct/with-proto doer-defaults)
                          (struct/proto-flatten)))
    ~(upscope
       (def doer ,doer-name)
       (def description ,description)
       ,;(seq [[k v] :pairs spec-struct]
           ~(def ,(symbol k) ,v))))

  # A helper requires certain properties. They are derived by adding -helper on
  # to the keys in this prototype struct
  (def- helper-defaults {:name-is nil
                         :mandatory-props {}
                         :optional-props {}})

  (defmacro when-defined
    "Expand to body only if sym is already defined"
    [sym & body]
    (when (get (curenv) sym)
      ~(do ,;body)))

  # Takes a raw spec, returns a struct
  (defn enrich-spec
    [spec attrs]
    (->> ;spec
         (table)
         (spec-with-defaults (extract-defaults (attrs :mandatory-props)))
         (expand-from-value)))

  (defmacro collect-attributes
    [suffix]
    {:mandatory-props (symbol "mandatory-props-" suffix)
     :optional-props (symbol "optional-props-" suffix)})

  (defmacro make-ensure-fn
    [name]
    ~(defn ensure
       [name & spec]
       (when-defined exactly-one-of
         (print "GIBBUS"))
       (def attrs (collect-attributes :ensure))
       (pinpoint-error
         :ensure
         (-> spec
             (enrich-spec attrs)
             (validate-spec attrs))
         (collector/push :ensure doer (make-ensure-resource)))))

  (defmacro make-ensure-fn!
    "Generates a boilerplate ensure function"
    [name]
    ~(defn ensure
       [name & spec]
       (pinpoint-error
         :ensure
         (when-defined exactly-one-of
           (if (not (has-exactly-one-of? exactly-one-of spec))
             (errorf "need exactly one of %s" (comma-sep exactly-one-of))))
         (when-defined at-least-one-of
           (if (not (has-at-least-one-of? at-least-one-of spec))
             (errorf "need at-least one of %s" (comma-sep at-least-one-of))))
         (collector/push :ensure doer (make-ensure-resource)))))

  (defmacro make-remove-fn
    "Generates a boilerplate remove function"
    [name]
    ~(defn remove
       [name & spec]
       (collector/push :remove doer (make-remove-resource))))

  (defmacro make-helper-fn
    "Generates a boilerplate helper function"
    [helper-name &named without-name]
    (with-syms [$mandatory-props $optional-props $defaults $exactly-one-of
                $at-least-one-of]
      (def $mandatory-props (symbol (string "mandatory-props-" helper-name)))
      (def $optional-props (symbol (string "optional-props-" helper-name)))
      (def $exactly-one-of (symbol (string "exactly-one-of-" helper-name)))
      (def $at-least-one-of (symbol (string "at-least-one-of-" helper-name)))
      ~(defn ,(symbol helper-name)
         ,(if without-name
            '[& spec]
            '[name & spec])
         ,(when without-name '(def name "NO-NAME"))
         (pinpoint-error
           ,(keyword helper-name)
           (when-defined ,$exactly-one-of
             (if (not (has-exactly-one-of? ,$exactly-one-of spec))
               (errorf "need exactly one of %s" (comma-sep ,$exactly-one-of))))
           (when-defined ,$at-least-one-of
             (if (not (has-at-least-one-of? ,$at-least-one-of spec))
               (errorf "need at-least one of %s" (comma-sep ,$at-least-one-of))))
           (let [user-spec ,(if without-name
                              '(make-spec-struct ;spec)
                              '(make-spec-struct :name name ;spec))
                 expanded-spec (spec-with-defaults
                                 (extract-defaults ,$mandatory-props)
                                 user-spec)
                 spec-struct (checked-spec expanded-spec
                                           ,$mandatory-props
                                           ,$optional-props)]

             (struct ,(keyword helper-name) spec-struct))))))

  (defn expand-list-struct
    "If any keys of strct are arrays or tuples, expand each key into a new entry
   with the same value. Returns a table. May clobber existing keys!"
    [strct]
    (table
      ;(catseq [[src dest] :pairs strct]
         (if (or (tuple? src) (array? src))
           (mapcat |(tuple $ dest) src)
           [src dest]))))

  (defn safe-val
    "Turn a string into something safe to put into a zone attr."
    [val]
    (def contains-peg (peg/compile ~(any (+ (* ($) :W) 1))))

    (def escape-peg
      (peg/compile
        ~{:special (+ (/ (set "\"") "\\\"")
                      (/ (set "\\") "\\\\")
                      (/ (set "\n") "\\n")
                      (/ (set "\t") "\\t")
                      (/ (set "\r") "\\r"))
          :normal (if-not :special (capture 1))
          :main (any (+ :special :normal))}))

    (if (empty? (peg/match contains-peg val))
      val
      (string "\"" (string/join (peg/match escape-peg val) "") "\""))))
