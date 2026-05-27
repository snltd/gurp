(def protected-keys
  "Trying to set any of these keys in metadata will result in an error"
  [:host])

(defn new-metadata
  "Returns an empty *metadata*."
  []
  @{})

(var *metadata* (new-metadata))

(defn metadata
  "Adds to the *metadata* struct"
  [key value]
  (if (has-value? protected-keys key)
    (errorf "protected metadata key: %s" key))

  (if (= (get *metadata* key :__undefined) :__undefined)
    (set (*metadata* key) value)
    (errorf "duplicate metadata key: %v" key)))
