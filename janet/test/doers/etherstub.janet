(use judge)
(use ./test-lib)
(import ../../src/doers/etherstub)
(use ../../src/collector)

(deftest etherstub
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "etherstub" (curenv))
  (etherstub/ensure "estub1")

  (test *collector*
    @{:ensure @{:etherstub @[{:_id "/test-role/etherstub/newstub0"
                              :name "newstub0"
                              :role "test-role"}
                             {:_id "/test-role/etherstub/estub1"
                              :name "estub1"
                              :role "test-role"}]}
      :remove @{:etherstub @[{:_id "/test-role/etherstub/oldstub0"
                              :name "oldstub0"
                              :role "test-role"}]}}))

(deftest etherstub-error
  (test-error
    (etherstub/ensure "estub4"
                      :with "field")
    "unexpected property :with. Valid properties are :label"))
