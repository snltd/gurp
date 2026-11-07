# Return the given array without the given element. Like a non-destructive
# array/remove
(defn drop-element [arr index]
  (if (>= index (length arr))
    arr
    (array/join (array/slice arr 0 index) (slice arr (inc index)))))

# Turn an array of arrays into a struct
(defn tabular-data->struct [array-of-arrays]
  (->> array-of-arrays (flatten) (splice) (struct)))

# Turn a string into a downcased keyword, and remove #s
(defn ->key [key]
  (->> key (string/replace "#" "") (string/ascii-lower) (keyword)))

# If value is a number, return it as a :number, otherwise, return as-is
(defn parsed-value [value]
  (if-let [num (scan-number value)] num value))

# Returns a flat array of values, whatever type of values it's given
(defn values-as-tuple [& values]
  (flatten (array values)))

# Remove empty elements from an array. Errors if any element is not lengthable?
(defn drop-empty [arr]
  (filter |(not (empty? $)) arr))

# Remove nil elements from an array
(defn compact [arr]
  (filter |(not (nil? $)) arr))
