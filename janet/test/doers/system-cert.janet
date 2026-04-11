(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/system-cert)

(deftest 
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "system-cert" (curenv))

  (test *collector*)))

(deftest system-cert-error
  (test-error
    (system-cert/ensure "missing-source"))

  (test-error
    (system-cert/ensure "bad-property" :oops "wat")))
