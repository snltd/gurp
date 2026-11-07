(import ../../collector)
(import ../lib/lib :only [safe-val])
(use ../lib/definers)

(defn completer [spec-table]
  # If there's a value without a atype, set the type
  (if (and (has-key? spec-table :value) (not (has-key? spec-table :type)))
    (set (spec-table :type)
         (match (type (spec-table :value))
           :number "uint"
           :boolean "boolean"
           _ "string")))

  # If there's a string value, make it safe
  (if-let [value (spec-table :value)]
    (match (spec-table :type)
      "astring" (set (spec-table :value) (string value))
      "string" (set (spec-table :value) (lib/safe-val value))))

  spec-table)

(defhelper :zone :attr
  "Set attributes on a zone being created by the zone doer."
  :name-is "Attribute name"
  (helper
    :complete-with completer
    (property "name" [:string] :mandatory
              "Attribute name. Derived from resource name and not specified
               as a property")
    (property "value" [:string :boolean :number] :mandatory "Attribute value")
    (property "type" [:string] :mandatory "The type of the value")))

(make-helper-fn :attr)
