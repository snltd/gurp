(use ../lib)

(defhelper :zone :attr
  "Set attributes on a zone being created by the zone doer."
  :name-is "Attribute name"

  :optional-props
  {:type {:types [:string]
          :help "The type of the value. Gurp will take a pretty good guess though"}}

  :mandatory-props
  {:value {:types [:string :boolean :number]
           :help "Attribute value"}
   :name {:types [:string]
          :help "Attribute name. Derived from resource name"}})

(defn attr
  "Given a spec, return a zone attr struct."
  [name & spec]
  (let [spec-struct (make-spec-struct :name name ;spec)
        expanded-spec (spec-with-defaults defaults-attr spec-struct)
        spec-table (pinpoint-error :attr
                                   (checked-spec expanded-spec
                                                 mandatory-props-attr
                                                 optional-props-attr))]

    (if-not (has-key? spec-table :type)
      (set (spec-table :type)
           (match (type (spec-table :value))
             :number "uint"
             :boolean "boolean"
             _ "string")))

    (match (spec-table :type)
      "astring" (set (spec-table :value) (string (spec-table :value)))
      "string" (set (spec-table :value) (safe-val (spec-table :value))))

    (struct :attr spec-table)))
