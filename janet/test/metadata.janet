(use judge)
(use ../src/gurp)

(deftest metadata
  (set *metadata* (new-metadata))
  (host "test"
        (metadata :item-1 "first")
        (metadata :item-2 "second"))

  (test
    ((machine-config) :metadata)
    {:item-1 "first"
     :item-2 "second"
     :name "test"}))

(deftest metadata-duplicate-key
  (set *metadata* (new-metadata))
  (metadata :item-1 "first")
  (test-error (metadata :item-1 "second")
              "duplicate metadata key: :item-1"))
