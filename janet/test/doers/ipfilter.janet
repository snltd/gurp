(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/ipfilter)

(deftest ipfilter
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ipfilter" (curenv))

  (test *collector*
    @{:ensure @{:ipfilter @[{:_id "/test-role/ipfilter/rules-from-file"
                             :always-reload false
                             :from "test/ipfilter-test"
                             :name "rules-from-file"
                             :priority 1
                             :role "test-role"}
                            {:_id "/test-role/ipfilter/rules-in-config"
                             :always-reload true
                             :content "block in log all\nblock out all"
                             :name "rules-in-config"
                             :priority 0
                             :role "test-role"}]}
      :remove @{:ipfilter @[{:_id "/test-role/ipfilter/removes-all-rules"
                             :name "removes-all-rules"
                             :role "test-role"}]}}))

(deftest ipfilter-errors
  (test-error
    (ipfilter/ensure "error-test-1" :from "test/ipfilter")
    "did not find mandatory property :priority. Mandatory properties are :priority")

  (test-error
    (ipfilter/ensure "error-test-2" :priority 0)
    "need exactly one of :content or :from"))
