(use ../dsl)
(use ../../test/doers/lib/test-lib)

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
  (seq [doer :in (sorted (os/dir (doer-root)))
        :when (string/has-suffix? ".janet" doer)
        :unless (= "lib.janet" doer)]
    (string/replace ".janet" "" doer)))

(defn lookup
  "Return the value of the given prop in doer or doer/helper"
  [resource prop &opt helper]
  (try
    (eval (symbol (a/b resource (if helper (keyword prop :- helper) prop))))
    ([_] nil)))
    
(defn squeeze
  "Squash repeated whitespace into a single space"
  [str]
  (peg/replace-all '(* (some :s)) " " str))
