(use ../lib)

(def description-attr "Set attributes on a zone being created by the zone doer.")
(def name-is-attr "Attribute name")
(def optional-props-attr
  {:type
   {:types [:string]
    :help "The type of the value. Gurp will take a pretty good guess though"}})
(def mandatory-props-attr
  {:value
   {:types [:string :boolean :number]
    :help "Attribute value"}
   :name
   {:types [:string]
    :help "Attribute name. Derived from resource name"}})
(def defaults-attr {})

(defn attr
  "Given a spec, return a zone attr struct."
  [name & spec]

  (def spec-struct (make-spec-struct :name name ;spec))
  (def spec-table (checked-spec (spec-with-defaults defaults-attr spec-struct)
                                mandatory-props-attr
                                optional-props-attr))

  (if-not (has-key? spec-table :type)
    (set (spec-table :type)
         (match (type (spec-table :value))
           :number "uint"
           :boolean "boolean"
           _ "string")))

  (if (= "astring" (spec-table :type))
    (set (spec-table :value) (string (spec-table :value))))

  (struct :attr spec-table))
