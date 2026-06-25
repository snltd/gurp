#
# Functions, macros, and bindings required by the doer modules in this directory.
# 
(import ../dsl :only [qualify-from-path])
(import ../collector)

(def client-api-version "v1")

(def ip-protocols
  "Protocols supported by ipadm"
  [:ip :ipv4 :ipv6 :icmp :tcp :sctp :udp])

(def protocol-opts
  (tabseq [protocol :in ip-protocols]
    protocol {:types [:struct :table]
              :help (string/format "key-value pairs of valid %s properties"
                                   protocol)}))

(defn comma-sep
  "Return a comma-separated string of the items in list"
  [list]
  (string/join (map |(string/format "%p" $) list) ", "))

(defn check-key-type
  "Checks something is of a permissible type. Raises an error if it is not.
  Values can *always* be keywords, because they denote references."
  [prop-name prop-value allowed-types]
  (let [prop-type (type prop-value)]
    (if-not (or (= prop-type :keyword) (has-value? allowed-types prop-type))
      (errorf "%s is of type %v. Allowed types %s"
              prop-name
              prop-type
              (comma-sep allowed-types)))))

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

(defn checked-spec
  "Compares a user's spec against what a resource definition expects. Raises
  an error if anything is not as it should be, otherwise returns the given spec
  as a struct."
  [spec-struct mandatory-props optional-props]

  (def optional-props
    (merge {:label {:types [:string]
                    :help "Optional label"}}
           optional-props))

  (loop [[prop-name prop-spec] :pairs mandatory-props]
    (let [prop-value (get spec-struct prop-name)]
      (if (nil? prop-value)
        (errorf "did not find mandatory property %p. Mandatory properties are %s"
                prop-name
                (comma-sep (keys mandatory-props)))
        (check-key-type prop-name prop-value (prop-spec :types)))))

  (loop [[prop-name prop-value] :pairs spec-struct]
    (if-not (has-key? mandatory-props prop-name) # we've already checked these
      (if-let [prop-spec (get optional-props prop-name)]
        (check-key-type prop-name prop-value (prop-spec :types))
        (errorf "unexpected property %p. Valid properties are %s"
                prop-name
                (comma-sep (array/concat @[]
                                         (keys mandatory-props)
                                         (keys optional-props)))))))
  spec-struct)

(defn resource-id
  "Safely generate a resource ID"
  [resource-type role name]
  (let [name (if (nil? name)
               "NO-NAME"
               (string/replace-all "/" "_" name))]

    (string/format "/%s/%s/%s" role resource-type name)))

(defn spec->resource
  "Decorate a spec struct with everything it needs to become a resource."
  [resource-type resource-name spec-struct]
  (let [role (dyn :role-dyn "NO-ROLE")
        final-id-chunk (get spec-struct :label resource-name)]

    (table/to-struct
      (merge {:_id (resource-id resource-type role final-id-chunk)
              :name resource-name
              :role role}
             spec-struct))))

(defn spec-with-defaults
  "Merge defaults with user values. We don't use prototypes now"
  [default-prop-values spec-struct]
  (merge default-prop-values spec-struct))

(defmacro pinpoint-error
  "Wraps an error in a string describing the resource which caused it"
  [action & body]
  (with-syms [$e]
    ~(try
       (do
         ,;body)
       ([$e]
         (errorf "In %s/%s %s: %s" doer ,action name $e)))))

(defmacro make-ensure-resource
  "Pulls together some boilerplate in doer ensure functions"
  []
  (with-syms [$e]
    ~(pinpoint-error
       :ensure
       (let [spec-struct (make-spec-struct ;spec)
             all-specs (spec-with-defaults defaults-ensure spec-struct)
             safe-specs (checked-spec all-specs
                                      mandatory-props-ensure
                                      optional-props-ensure)]

         (spec->resource doer name safe-specs)))))

(defmacro make-remove-resource
  "Pulls together some boilerplate in doer remove functions"
  []
  ~(pinpoint-error
     :remove
     (let [spec-struct (make-spec-struct ;spec)
           all-specs (spec-with-defaults defaults-remove spec-struct)
           safe-specs (checked-spec all-specs
                                    mandatory-props-remove
                                    optional-props-remove)]

       (spec->resource doer name safe-specs))))

(defn has-exactly-one-of?
  "Checks whether a spec contains exactly one of the required-keys"
  [required-keys spec]
  (one?
    (length
      (filter |(has-value? required-keys $) (keys (make-spec-struct ;spec))))))

(defn key-has-value?
  "Checks whether the given key has an acceptable value"
  [spec key acceptable-values]
  (if-let [given-value (get (struct ;spec) key)]
    (if-not (has-value? acceptable-values given-value)
      (errorf "%q must be one of %s [got '%s']"
              key
              (comma-sep acceptable-values)
              given-value))))


(defmacro table->flat-tuple
  "Completely flattens a struct or table, including its keys"
  [table]
  ~(flatten (pairs ,table)))

(defmacro expand-resource
  "Group the results of in-resource functions like (zone-fs) into a list under
  a single key. Partitions `modified-spec` items whose keys do or do not match
  the given `key`. The matches ('is' group) are flattened into a single array
  (or, if `:as-struct` is passed, reduced to the first matching struct) and
  stored under `key`. Non-matching items ('is-not' group) are preserved."
  [key &keys {:as-struct as-struct}]
  (with-syms [$is-key $key-list $vals]
    ~(do
       (let [$is-key
             (group-by
               |(and (struct? $) (deep= @[,key] (keys $)))
               modified-spec)]

         (if-let [$key-list ($is-key true)]
           (let [$vals (mapcat values $key-list)]
             (set modified-spec
                  (tuple
                    ;(get $is-key false @[])
                    ,key
                    (if ,as-struct (first $vals) $vals)))))))))

(defn expand-svc-property
  "Turns a svcprop value into a struct describing a typed value"
  [value]
  (match (type value)
    :keyword value
    :number {:type "integer" :value value}
    :boolean {:type "boolean" :value value}
    _ {:type "astring" :value value}))

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

    (if-let [protocol-parts (get parts 0)
             protocol-table (struct ;protocol-parts)]
      (set (spec-table :protocols) protocol-table))

    spec-table))

(defn- server-url
  [server-name from-path]
  (string "http://" server-name "/" client-api-version "/file/" from-path))

(defn expand-from-value
  "Used by any resource that takes a :from. e.g. file and system-cert. Give it
  a spec table, and it will turn relative :from paths into fully qualified paths
  or server references, depending on the current execution environment. Returns
  a modified spec-table."
  [spec-table]
  (if-let [from-path (spec-table :from)]
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
                     :defaults-ensure {}
                     :defaults-remove {}})

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
                       :optional-props {}
                       :defaults {}})

(defmacro defhelper
  "Creates the defs required for a helper"
  [doer-name helper-name description & spec]
  (def spec-struct (->> ;spec
                        (struct/with-proto helper-defaults)
                        (struct/proto-flatten)))

  ~(upscope
     (def doer ,doer-name)
     (def ,(symbol (string "description-" helper-name)) ,description)
     ,;(seq [[k v] :pairs spec-struct]
         ~(def ,(symbol (string k "-" helper-name)) ,v))))

(defmacro defensure
  "Generates a boilerplate ensure function"
  [name]
  ~(defn ensure
     [name & spec]
     (collector/push :ensure doer (make-ensure-resource))))

(defmacro defremove
  "Generates a boilerplate remove function"
  [name]
  ~(defn remove
     [name & spec]
     (collector/push :remove doer (make-remove-resource))))
