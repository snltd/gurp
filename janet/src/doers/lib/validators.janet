(import ./lib :only [keys-in-struct comma-sep])

(defn check-key-value
  "Compares a given value against a list of permissible values. If the list is
   nil, which it usually is, anything is allowed"
  [property value possible-values]
  (when (and (not (nil? possible-values))
             (not (has-value? possible-values value)))
    (errorf "%s must be one of %s [Got '%v']"
            property
            (lib/comma-sep possible-values)
            value))
  true)

(defn check-key-type
  "Checks something is of a permissible type. Raises an error if it is not.
  Values can *always* be keywords, because they denote references."
  [prop-name prop-value allowed-types]
  # (print (string/format "prop-name %p  prop-value %p  allowed-types %p " prop-name prop-value allowed-types) )
  (let [prop-type (type prop-value)]
    (if-not (or (= prop-type :keyword)
                (has-value? allowed-types prop-type)
                (and (= :table prop-type) (has-value? allowed-types :struct))
                (and (= :array prop-type) (has-value? allowed-types :tuple)))
      (errorf "%s is of type %v. Allowed types %s"
              prop-name
              prop-type
              (lib/comma-sep allowed-types)))))

(defn list-props
  "Returns a comma-separated list of property names, omitting :name"
  [prop-list]
  (lib/comma-sep (sorted (filter |(not= $ :name) prop-list))))

(defn- check-key
  [prop-name prop-value prop-spec]
  (check-key-type prop-name prop-value (prop-spec :types))
  (check-key-value prop-name prop-value (prop-spec :values)))

(defn checked-spec
  "Compares a user's spec against what a resource definition expects. Raises
  an error if anything is not as it should be, otherwise returns the given spec
  as a struct."
  [action-attrs spec-struct]
  (def ignore-props [:from-helper :unique])

  (let [mandatory-props (get-in action-attrs [:props :mandatory] {})
        optional-props (merge-into @{:label {:types [:string]
                                             :help "Optional label"}}
                                   (get-in action-attrs [:props :optional] {}))]

    (loop [[prop-name prop-spec] :pairs mandatory-props]
      (let [prop-value (get spec-struct prop-name)]
        (if-not (nil? prop-value)
          (check-key prop-name prop-value prop-spec)
          (errorf
            "did not find mandatory property %p. Mandatory properties are %s"
            prop-name
            (list-props (keys mandatory-props))))))

    (loop [[prop-name prop-value] :pairs spec-struct
           :when (not (or (has-key? mandatory-props prop-name) # already checked 
                          (has-value? ignore-props prop-name)))]
      (if-let [prop-spec (get optional-props prop-name)]
        (check-key prop-name prop-value prop-spec)
        (errorf "unexpected property %p. Valid properties are %s"
                prop-name
                (list-props (array/concat @[]
                                          (keys mandatory-props)
                                          (keys optional-props))))))
    spec-struct))

(defn has-at-least-one-of?
  [required-keys spec-struct]
  (not (empty? (lib/keys-in-struct required-keys spec-struct))))

(defn has-exactly-one-of?
  [required-keys spec-struct]
  (one? (length (lib/keys-in-struct required-keys spec-struct))))

(defn validate-spec
  [action-attrs spec]
  (when-let [props (action-attrs :exactly-one-of)]
    (when (not (has-exactly-one-of? props spec))
      (errorf "need exactly one of %s" (lib/comma-sep props))))
  (when-let [props (action-attrs :at-least-one-of)]
    (when (not (has-at-least-one-of? props spec))
      (errorf "need at-least one of %s" (lib/comma-sep props))))
  (checked-spec action-attrs spec))
