(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/system-cert)

(setdyn :gurp-config-root "/gurpdir")

(deftest
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "system-cert" (curenv))

  (test *collector*
    @{:ensure @{:system-cert @[{:_id "/NO-ROLE/system-cert/from-file"
                                :from "/gurpdir/files/ca/example"
                                :name "from-file"
                                :role "NO-ROLE"}
                               {:_id "/NO-ROLE/system-cert/from-url"
                                :from-url "https://cert-service/api"
                                :name "from-url"
                                :role "NO-ROLE"}]}
      :remove @{:system-cert @[{:_id "/NO-ROLE/system-cert/unwanted-cert"
                                :name "unwanted-cert"
                                :role "NO-ROLE"}]}}))

(deftest system-cert-error
  (test-error
    (system-cert/ensure "missing-source")
    "In system-cert/ensure missing-source: Provide exactly one of :content, :from, :from-url")

  (test-error
    (system-cert/ensure "bad-property" :oops "wat")
    "In system-cert/ensure bad-property: Provide exactly one of :content, :from, :from-url"))
