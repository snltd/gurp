# Internal functions 

(defn drop-element
  "Return the given array without the given element. Like a non-destructive
   array/remove"
  [arr index]
  (if (>= index (length arr))
    arr
    (array/join (array/slice arr 0 index) (slice arr (inc index)))))

(defn tabular-data->struct
  "Turn an array of arrays into a struct"
  [array-of-arrays]
  (->> array-of-arrays
       (flatten)
       (splice)
       (struct)))

(defn ->key
  "turn a string into a downcased keyword, and remove #s"
  [key]
  (->> key (string/replace "#" "") (string/ascii-lower) (keyword)))

(defn parsed-value
  "If value is a number, return it as a :number, otherwise, return as-is"
  [value]
  (if-let [num (scan-number value)] num value))

(defn values-as-tuple
  "Returns a flat array of values, whatever type of values it's given"
  [values]
  (flatten (array values)))

(defn compact
  "Remove empty elements from an array"
  [vector]
  (filter |(not (empty? $)) vector))
