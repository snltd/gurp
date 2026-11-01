(use judge)
(use ../../src/doer-docs/lib)

(deftest lookup
  (import ../../src/doers/directory)
  (import ../../src/doers/zone)

  (test
    ((lookup :directory :attrs) :description)
    "Create and remove directories. Parents are created like `mkdir -p`, but with  the owner/group/mode of the gurp process. Removal always removes directory  contents.")

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
