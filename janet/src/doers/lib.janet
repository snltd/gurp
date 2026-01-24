#
# Functions and bindings required by the doer modules in this directory.
# 
(def ip-protocols
  "Protocols supported by ipadm"
  [:ip :ipv4 :ipv6 :icmp :tcp :sctp :udp])

(def protocol-opts
  (tabseq [protocol :in ip-protocols]
    protocol {:types [:struct :table]
              :help (string "key-value pairs of valid " protocol " properties")}))

(defn comma-sep
  "Return a comma-separated string of the items in list"
  [list]
  (string/join (map |(string/format "%p" $) list) ", "))

(defn check-key-type
  "Checks something is of a permissible type. Raises an error if it is not. Values
  can *always* be keywords, because they denote references."
  [prop-name prop-value allowed-types]
  (def prop-type (type prop-value))

  (if-not (or (= prop-type :keyword) (has-value? allowed-types prop-type))
    (error
      (string/format "%s is of type %v. Allowed types %s"
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
      (error
        (string/format "unable to create struct from %d arg(s):  %p: %s" (length spec) spec e)))))

(defn checked-spec
  "Compares a user's spec against what a resource definiton expects. Raises
  an error if anything is not as it should be, otherwise the given spec as a
  struct."
  [spec-struct mandatory-props optional-props]

  (def optional-props
    (merge {:label {:types [:string]
                    :help "Optional label"}}
           optional-props))

  (loop [[prop-name prop-spec] :pairs mandatory-props]
    (def prop-value (get spec-struct prop-name))
    (if (nil? prop-value)
      (error
        (string/format
          "did not find mandatory property %p. Mandatory properties are %s"
          prop-name
          (comma-sep (keys mandatory-props))))
      (check-key-type prop-name prop-value (prop-spec :types))))

  (loop [[prop-name prop-value] :pairs spec-struct]
    (if-not (has-key? mandatory-props prop-name) # we've already checked these
      (if-let [prop-spec (get optional-props prop-name)]
        (check-key-type prop-name prop-value (prop-spec :types))
        (error
          (string
            (string/format "unexpected property %p. Valid properties are "
                           prop-name)
            (comma-sep (array/concat @[] (keys mandatory-props) (keys optional-props))))))))

  spec-struct)

(defn resource-id
  "Safely generate a resource ID"
  [resource-type role name]
  (def name (if (nil? name)
              "NO-NAME"
              (string/replace-all "/" "_" name)))

  (string/format "/%s/%s/%s" role resource-type name))

(defn spec->resource
  "Decorate a spec struct with everything it needs to become a resource."
  [resource-type resource-name spec-struct]
  (def role (dyn :role-dyn "NO-ROLE"))
  (def final-id-chunk (get spec-struct :label resource-name))

  (table/to-struct
    (merge {:_id (resource-id resource-type role final-id-chunk)
            :name resource-name
            :role role}
           spec-struct)))

(defn spec-with-defaults
  "Merge defaults with user values. We don't use prototypes now"
  [default-prop-values spec-struct]
  (merge default-prop-values spec-struct))

(defmacro make-ensure-resource
  "Pulls together some boilerplate in doer ensure functions"
  []
  ~(do
     (def spec-struct (make-spec-struct ;spec))
     (def all-specs (spec-with-defaults defaults-ensure spec-struct))
     (def safe-specs (checked-spec all-specs
                                   mandatory-props-ensure
                                   optional-props-ensure))

     (spec->resource doer name safe-specs)))

(defmacro make-remove-resource
  "Pulls together some boilerplate in doer remove functions"
  []
  ~(do
     (def spec-struct (make-spec-struct ;spec))
     (def all-specs (spec-with-defaults defaults-remove spec-struct))
     (def safe-specs (checked-spec all-specs
                                   mandatory-props-remove
                                   optional-props-remove))

     (spec->resource doer name safe-specs)))

(defn has-exactly-one-of?
  "Checks whether a spec contains exactly one of the required-keys"
  [required-keys spec]
  (= 1
     (length
       (filter |(has-value? required-keys $) (keys (make-spec-struct ;spec))))))

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
  
  (def temp-spec-table
    (checked-spec (make-spec-struct ;spec) mandatory-props optional-props))

  (def parts (->>
               temp-spec-table
               (table->flat-tuple)
               (partition 2)
               (partition-by |(has-value? ip-protocols (first $)))
               (map flatten)))

  (def spec-table (table ;(get parts 1 [])))

  (if-let [protocol-parts (get parts 0)
           protocol-table (struct ;protocol-parts)]
    (set (spec-table :protocols) protocol-table))

  spec-table)
