(use judge)
(use ./lib/test-lib)
(use ../../src/collector)
(import ../../src/doers/ipfilter)
(setdyn :gurp-config-root "/gurpdir")

(deftest ipfilter
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ipfilter")

  (test *collector*
        @{:ensure @{:ipfilter @[{:_id "/test-role/ipfilter/rules-from-config"
                                 :always-reload true
                                 :content "block in log all\nblock out all"
                                 :name "rules-from-config"
                                 :priority 0
                                 :role "test-role"}
                                {:_id "/test-role/ipfilter/rules-from-file"
                                 :from "/gurpdir/files/test/ipfilter-test"
                                 :name "rules-from-file"
                                 :priority 1
                                 :role "test-role"}]}
          :remove @{:ipfilter @[{:_id "/test-role/ipfilter/removes-all-rules"
                                 :name "removes-all-rules"
                                 :role "test-role"}]}}))

(deftest ipfilter-error
  (test-error
    (ipfilter/ensure "error-test-1" :from "test/ipfilter")
    "In ipfilter/ensure error-test-1: did not find mandatory property :priority. Mandatory properties are :priority")

  (test-error
    (ipfilter/ensure "error-test-2" :priority 0)
    "In ipfilter/ensure error-test-2: need exactly one of :content, :from"))
