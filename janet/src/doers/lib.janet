#
# Functions required by the doer modules in this directory.
# 
(defn check-key-type
  "Checks something is of a permissible type. Raises an error if it is not"
  [prop-name prop-value allowed-types]
  (def prop-type (type prop-value))

  (if-not (has-value? allowed-types prop-type)
    (error
      (string/format "%s is of type %v. Allowed types: %s"
                     prop-name
                     prop-type
                     (string/join allowed-types ", ")))))

(defn make-spec-struct
  "Turn a list of property key/value pairs into a struct or, raise a more
  useful error than 'expected even number of arguments'"
  [& spec]
  (try
    (struct ;(flatten spec))
    ([e]
      (error
        (string/format "unable to create struct from %p: %s" spec e)))))

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
    (if-let [prop-value (get spec-struct prop-name)]
      (check-key-type prop-name prop-value (prop-spec :types))
      (error
        (string/format
          "did not find mandatory property '%s'. Mandatory propties are: %s"
          prop-name
          (string/join (keys spec-struct) ", ")))))

  (loop [[prop-name prop-value] :pairs spec-struct]
    (if-not (mandatory-props prop-name) # we've already checked these
      (if-let [prop-spec (get optional-props prop-name)]
        (check-key-type prop-name prop-value (prop-spec :types))
        (error
          (string
            (string/format "unexpected property '%s'. Valid properties are: "
                           prop-name)
            (string/join
              (array/concat @[] (keys mandatory-props) (keys optional-props))
              ", "))))))

  spec-struct)

(defn resource-id
  "Safely generate a resource ID"
  [resource-type role name]
  (string/format "/%s/%s/%s"
                 role
                 resource-type
                 (string/replace-all "/" "_" name)))

(defn spec->resource
  "Decorate a spec struct with everything it needs to become a resource."
  [resource-type resource-name spec-struct]
  (def role (dyn :role-dyn "NO-ROLE"))

  (merge {:_id (resource-id resource-type role resource-name)
          :name resource-name
          :role role}
         spec-struct))

(defn spec-with-defaults
  "Merge defaults with user values. We don't use prototypes now"
  [default-prop-values spec-struct]
  (merge default-prop-values spec-struct))

(defmacro make-resource
  "Pulls together some boilerplate in doer ensure/remove functions"
  []
  ~(do
     (def spec-struct (make-spec-struct spec))
     (def all-specs (spec-with-defaults default-prop-values spec-struct))
     (def safe-specs (checked-spec all-specs
                                   mandatory-ensure-props
                                   optional-ensure-props))

     (spec->resource doer name safe-specs)))
