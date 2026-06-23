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

(deftest parsed-value
  (test (parsed-value "123") 123)
  (test (parsed-value "abc") "abc")
  (test (parsed-value "abc123") "abc123")
  (test (parsed-value "123abc") "123abc"))

(deftest values-as-tuple
  (test (values-as-tuple [1 [2 3 4] [[5]]]) @[1 2 3 4 5])
  (test (values-as-tuple "merp") @["merp"])
  (test (values-as-tuple 1 2 3 4 5) @[1 2 3 4 5]))

(deftest drop-empties
(test (drop-empties []) @[])
(test (drop-empties ["a" "" "b" "c" ""]) @["a" "b" "c"]))

(deftest compact
 (test (compact [nil 1 2 nil 3 4 nil nil nil 5]) @[1 2 3 4 5])
  (test (compact []) @[])
(test (compact [1 2 3]) @[1 2 3]))
