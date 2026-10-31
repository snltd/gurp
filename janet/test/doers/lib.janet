(use judge)
(use ../../src/doers/lib)

(deftest expand-svc-property
  (test
    (expand-svc-property {:application/active true
                          :application/datadir "/data"
                          :application/timeout 50})
    @{:application/active {:type "boolean" :value true}
      :application/datadir {:type "astring" :value "/data"}
      :application/timeout {:type "integer" :value 50}}))

(deftest comma-sep
  (test (comma-sep [:merp :byerp :gurp]) ":merp, :byerp, :gurp")
  (test (comma-sep ["merp" "byerp" "gurp"]) "\"merp\", \"byerp\", \"gurp\"")
  (test (comma-sep ["merp"]) "\"merp\"")
  (test (comma-sep []) ""))

(deftest keys-in-struct
  (test (keys-in-struct [:a :b :c] {:a 1 :A 2 :alpha 3}) @[:a])
  (test (keys-in-struct [:a :b :c] {:a 1 :b 2}) @[:b :a])
  (test (keys-in-struct [:x :y :z] {:a 1 :b 2 :c 3}) @[]))

(deftest safe-val
  (test (safe-val "word") "word")
  (test (safe-val "two words") "\"two words\"")
  (test (safe-val `"Quote this!", I said`) "\"\\\"Quote this!\\\", I said\""))
