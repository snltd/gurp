(use ../user-helpers)

(defn a/b
  "Join the given words with slashes"
  [& words]
  (string/join words "/"))

(defn join-lines
  "Join the given lines with newlines"
  [lines]
  (string/join lines "\n"))

(defn term-width
  "Gurp sets the width of the user terminal as a dyn"
  []
  (dyn :term-width 80))

(defn repo-root
  "Return the root of this Gurp repo. Can by set dynamically by doers/build.rs"
  []
  (if-let [from-gurp (dyn :repo-root)]
    from-gurp
    (->> (dyn *current-file*)
         (os/realpath)
         (peg/replace '(* "/janet" (some 1)) ""))))

(defn doer-root
  "Return the directory containing front-end doer code"
  []
  (pathcat (repo-root) "janet/src/doers"))

(defn doers
  "Return array of all doer names"
  []
  (seq [doer :in (os/dir (doer-root))]
    (string/replace ".janet" "" doer)))

(defn doer-lookup
  "Fetch the given binding from the given doer definition file"
  [doer binding]
  (try
    (do
      (def lookup (symbol (a/b doer binding)))
      (eval lookup))
    ([_] nil)))

(defn subresource-lookup
  "As doer-lookup but for subresources"
  [doer subresource binding]
  (def lookup (symbol (string doer "/" binding "-" subresource)))
  (eval lookup))
