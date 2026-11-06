(use judge)
(use ../../../src/doers/lib/validators)

(deftest check-key-value
  (test (check-key-value "prop1" "any old thing" nil) true)
  (test (check-key-value "prop2" "ok" ["ok" "cool" "fine"]) true)
  (test-error (check-key-value "prop2" "nasty" ["ok" "cool" "fine"])
              "prop2 must be one of \"ok\", \"cool\", \"fine\" [Got '\"nasty\"']"))

(deftest check-key-type
  (test (check-key-type :name 123 [:number :string]) nil)
  (test (check-key-type :name "merp" [:number :string]) nil)
  (test-error
    (check-key-type :name 123 [:string])
    "name is of type :number. Allowed types :string"))

(deftest has-exactly-one-of?
  (test (has-exactly-one-of? [:a :b :c] {:a 1 :A 2 :alpha 3}) true)
  (test (has-exactly-one-of? [:a :b :c] {:a 1 :b 2 :c 3}) false)
  (test (has-exactly-one-of? [:x :y :z] {:a 1 :b 2 :c 3}) false))
