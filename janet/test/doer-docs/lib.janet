(use judge)
(use ../../src/doer-docs/lib)
(import ../commands :only [matches] :prefix "")

(deftest lookup
  (import ../../src/doers/directory)
  (import ../../src/doers/zone)

  (test
    (matches
      ((lookup :directory :attrs) :description)
      "Create and remove directories"
      "Removal always removes directory")
    2)

  (test
    (lookup :zone :description-fs)
    "Define a filesystem mapping when creating a zone.")

  (test
    (lookup :wat :nothing)
    nil))

(deftest squeeze
  (test (squeeze "Correctly spaced words") @"Correctly spaced words")
  (test (squeeze `  words  with     irregular
   spacing`)
        @" words with irregular spacing"))
