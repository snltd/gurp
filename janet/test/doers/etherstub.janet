(use judge)
(use ./_helpers)
(import ../../src/doers/etherstub)
(use ../../src/collector)

(deftest etherstub
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "etherstub" (curenv))
  (etherstub/ensure "estub1")

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

(deftest etherstub-error
  (test-error
    (etherstub/ensure "estub4"
                      :with "field")
    "unexpected property :with. Valid properties are :label"))
