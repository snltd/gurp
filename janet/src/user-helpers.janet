(defn argcat
  "Joins arguments to make a command"
  [& chunks]
  (string/join (tuple ;chunks) " "))

(defn pathcat
  "Joins tokens to make a path"
  [& chunks]
  (->
    (map |(string/trim $ "/") (tuple "" ;chunks))
    (string/join "/")))

(defn zfscat
  "Joins tokens to make a ZFS dataset name"
  [& chunks]
  (if (nil? chunks)
    (error "zfscat called with a nil"))
  (->
    (map |(string/trim $ "/") (tuple ;chunks))
    (string/join "/")
    (string/trim "/")))
