#
# Resolve references in resource definitions.
# 
(defn- resolve-reference
  "Recursively chase down a reference. Catches circular and dangling refs"
  [target-ref all-resources seen]
  (let [last-sep-index (last (string/find-all "/" target-ref))
        chunks (string/split "/" target-ref last-sep-index)
        resource-id (first chunks)
        property (keyword (last chunks))
        referenced-struct (find |(= resource-id (get $ :_id)) all-resources)]

    (if (nil? referenced-struct)
      (error
        (string/format "referenced resource not found: %s"
                       (keyword resource-id))))

    (if (has-key? seen resource-id)
      (error (string/format "detected circular reference: [%q]" seen)))

    (set (seen resource-id) true)
    (def referenced-val (referenced-struct property)) # could be another ref

    (if (keyword? referenced-val)
      (resolve-reference referenced-val all-resources seen)
      (if (nil? referenced-val)
        (error (string/format "referenced property is nil: %p" target-ref))
        referenced-val))))

(defn- resolve-references
  "Update any references in a resource with their final targets"
  [resource all-resources]
  (loop [[k v] :pairs resource]
    (cond
      (table? v)
      (set (resource k) (resolve-references v all-resources))
      (struct? v)
      (set (resource k) (table/to-struct
                          (resolve-references (struct/to-table v) all-resources)))
      (keyword? v)
      (set (resource k) (resolve-reference v all-resources @{}))))
  resource)

(defn resolve
  "resource-list is a list of resources of the same type"
  [resource-list all-resources]
  (map
    |(table/to-struct (resolve-references (struct/to-table $) all-resources))
    resource-list))
