(import ../../collector)
(use ../definers)

(defhelper :zone :attr
  "Set attributes on a zone being created by the zone doer."
  :name-is "Attribute name"
  (helper
    (property "name" [:string] :mandatory
              "Attribute name. Derived from resource name and not specified
               as a property")
    (property "value" [:string :boolean :number] :mandatory "Attribute value")
    (property "type" [:string] :mandatory "The type of the value")))

(make-helper-fn :attr)
