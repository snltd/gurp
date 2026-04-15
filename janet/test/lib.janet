(use judge)
(use ../src/lib)

(deftest drop-element
  (test (drop-element [10 11 12 13 14 15] 2) @[10 11 13 14 15])
  (test (drop-element [10 11 12 13 14 15] 0) @[11 12 13 14 15])
  (test (drop-element [10 11 12 13 14 15] 5) @[10 11 12 13 14])
  (test (drop-element [10 11 12 13 14 15] 6) [10 11 12 13 14 15]))


(deftest tabular-data->struct
  (test (tabular-data->struct
          [:key1 {:a 1 :b 2} :key2 {:a 10 :b 20}])
    {:key1 {:a 1 :b 2} :key2 {:a 10 :b 20}}))


(deftest ->key
  (test (->key "PascalCase") :pascalcase)
  (test (->key :nothing-to-change) :nothing-to-change))

(deftest parse-value
  (test (parsed-value "123") 123)
  (test (parsed-value "abc") "abc")
  (test (parsed-value "abc123") "abc123")
  (test (parsed-value "123abc") "123abc"))
