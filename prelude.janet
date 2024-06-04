# (defn fields 
#   "Return the given numbered fields from the given array"
#   [fields arr]
#   (tuple/slice (map |(get arr $) fields)))

(def rdir "/etc/passwd")

(defn ikey->val
  "Return two fields from arr. ifield will be an int, sfield a string"
  [ifield sfield arr]
  [(scan-number (get arr ifield)) (get arr sfield)])

(defn create-uid-map
  "create a struct of id (int) -> name (string)."
  [file field fields]
  (->>
    (slurp file)
    (string/split "\n")
    (map |(string/split ":" $))
    (filter |(= fields (length $)))
    (mapcat |(ikey->val 2 0 $))
    (splice)
    (table)
    (table/to-struct)))

(def uid->user (create-uid-map "/etc/passwd" 2 7))
(def user->uid (invert uid->user))
(def gid->group (create-uid-map "/etc/group" 1 4))
(def group->gid (invert gid->group))

# (assert (= "sys"))
# (pp uid->user)
# (pp gid->group)
