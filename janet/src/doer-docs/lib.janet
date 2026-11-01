(use ../dsl)
(use ../../test/doers/lib/test-lib)
(import ../lib :only [compact] :as gurp-lib)

(defn joint [& sections]
  (string/join (gurp-lib/compact (array ;sections)) "\n\n"))

(defn a/b [& words]
  (string/join words "/"))

(defn join-lines [lines]
  (string/join lines "\n"))

# Squash repeated whitespace into a single space
(defn squeeze [str]
  (peg/replace-all '(* (some :s)) " " str))

# Name is used internally, but shouldn't be exposed as a property in the docs
(defn props-without-name [action-info category]
  (def ret (get-in action-info [:props category] @{}))
  (set (ret :name) nil)
  ret)

(defn term-width []
  (dyn :term-width 80)) # Gurp sets the width of the user terminal as a dyn

# Return the root of this Gurp repo. Can by set dynamically by doers/build.rs
(defn repo-root []
  (if-let [from-gurp (dyn :repo-root)]
    from-gurp
    (->> (dyn *current-file*)
         (os/realpath)
         (peg/replace '(* "/janet" (some 1)) ""))))

# Directory containing front-end doer code
(defn doer-root []
  (pathcat (repo-root) "janet/src/doers"))

# Array of all doer names
(defn doers []
  (seq [doer :in (sorted (os/dir (doer-root)))
        :when (string/has-suffix? ".janet" doer)
        :unless (= "lib.janet" doer)]
    (string/replace ".janet" "" doer)))

# Value of the given prop in doer or doer/helper
(defn lookup [resource prop &opt helper]
  (try
    (eval (symbol (a/b resource (if helper (keyword prop :- helper) prop))))
    ([_] nil)))
