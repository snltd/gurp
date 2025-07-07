(use judge)
(use ../lib/gurp)

(deftest "test zone functions"
  (setdyn :role-dyn "test-role")
  (test
    (zone/ensure "test-zone"
    :brand "lipkg"
    )
  (test
    (zone/remove "defunct-zone")))
