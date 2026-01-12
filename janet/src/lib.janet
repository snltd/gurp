(defn compact
  "Remove empty elements from an array"
  [vector]
  (filter |(not (empty? $)) vector))

