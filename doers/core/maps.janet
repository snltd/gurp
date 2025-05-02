(var cache @{})

(defn extract-id-and-val-fields
  "Return two numbered fields from array of strings. ifield will be converted to an int, sfield left as a string"
  [arr int-field-idx string-field-idx]
  [(scan-number (get arr int-field-idx)) (get arr string-field-idx)])

(defn- map-from-file-fields
  "Return a struct of id (int) -> name (string)"
  [file int-field-idx string-field-idx expected-field-count]
  (->>
    (slurp file)
    (string/split "\n")
    (map |(string/split ":" $))
    (filter |(= expected-field-count (length $)))
    (mapcat |(extract-id-and-val-fields $ int-field-idx string-field-idx))
    (splice)
    (table)
    (table/to-struct)))

(defn generate-group-map 
  "Creates a map of group-id -> group-name"
  [file]
  (map-from-file-fields file 2 0 4))

(defn generate-passwd-map 
  "Creates a map of user-id -> user-name"
  [file]
  (map-from-file-fields file 2 0 7))

(defn uid->name 
  "Return a map of UID (int) to username (string)"
  [&opt file]
  (default file "/etc/passwd")
  (if-let [cached (cache :uid->name)]
    cached
    (do
      (put cache :uid->name (generate-passwd-map file))
      (uid->name file))))

(defn name->uid 
  "Return a map of username (string) to UID (int)"
  [&opt file]
  (default file "/etc/passwd")
  (if-let [cached (cache :name->uid)]
    cached
    (do
      (put cache :name->uid (invert (uid->name file)))
      (name->uid file))))

(defn gid->name 
  "Return a map of GID (int) to group name (string)"
  [&opt file]
  (default file "/etc/group")
  (if-let [cached (cache :gid->name)]
    cached
    (do
      (put cache :gid->name (generate-group-map file))
      (gid->name file))))

(defn name->gid 
  "Return a map of group name (string) to GID (int)"
  [&opt file]
  (default file "/etc/group")
  (if-let [cached (cache :name->gid)]
    cached
    (do
      (put cache :name->gid (invert (gid->name file)))
      (name->gid file))))
