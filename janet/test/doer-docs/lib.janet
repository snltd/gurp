(use judge)
(use ../../src/doer-docs/lib)

(deftest doer-lookup
  (import ../../src/doers/directory)
  (import ../../src/doers/zone)

  (test
    (doer-lookup :directory :description)
    "Create and remove directories. Parents are created like mkdir -p,                but with the owner/group/mode of the gurp process. Removal always                removes directory contents.")
  
  (test
    (doer-lookup :zone :description-fs)
    "Define a filesystem mapping when creating a zone.")

  (test
    (doer-lookup :wat :nothing)
    nil))
