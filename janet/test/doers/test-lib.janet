#
# Helpers for tests.
# 
(defn pathcat [& parts] (string/join parts "/"))

(def example-root
  (->>
    (dyn *current-file*)
    (os/realpath)
    (peg/replace '(* "test" (some 1)) "examples")))

(defn import-tests
  "Bring any doer examples into scope for Janet tests"
  [doer env]
  (loop [file :in (sort (os/dir (pathcat example-root doer)))]
    (dofile (pathcat example-root doer file) :env env)))
