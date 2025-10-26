(use judge)
(use ../lib/gurp)

(deftest "etherstub-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (etherstub/ensure "estub0")
  (etherstub/ensure "estub1")
  (etherstub/remove "estub2")

  (test *collector*
    @{:ensure @{:etherstub @[{:_id "/test-role/etherstub/estub0"
                              :name "estub0"
                              :role "test-role"}
                             {:_id "/test-role/etherstub/estub1"
                              :name "estub1"
                              :role "test-role"}]}
      :remove @{:etherstub @[{:_id "/test-role/etherstub/estub2"
                              :name "estub2"
                              :role "test-role"}]}}))

(deftest "etherstub-error"
  (test-error
    (etherstub/ensure "estub4"
                :with "field")
    "Failed to validate user input for etherstub 'estub4': etherstub 'estub4' has unrecognised key(s): with"))

