(defn argcat
  "Joins arguments to make a command"
  [& chunks]
  (string/join (tuple ;chunks) " "))
