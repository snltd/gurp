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
  [doer]
  (loop [file :in (sort (os/dir (pathcat example-root doer)))]
    (eval-string (slurp (pathcat example-root doer file)))))

(defn import-test
  "Bring a single doer example into scope for Janet tests"
  [file]
  (eval-string (slurp (pathcat example-root file))))
