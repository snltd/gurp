(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/ipnat)

(deftest ipnat
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ipnat" (curenv))

  (test *collector*
    @{:ensure @{:ipnat @[{:_id "/test-role/ipnat/rules-in-config"
                          :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\nrdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin"
                          :name "rules-in-config"
                          :priority 1
                          :role "test-role"}
                         {:_id "/test-role/ipnat/rules-in-file"
                          :from "test/ipnat-test"
                          :name "rules-in-file"
                          :priority 2
                          :role "test-role"}]}
      :remove @{:ipnat @[{:_id "/test-role/ipnat/removes-all-rules"
                          :name "removes-all-rules"
                          :role "test-role"}]}}))

(deftest ipnat-error
  (test-error
    (ipnat/ensure "error-test-1" :from "test/ipnat")
    "In ipnat/ensure error-test-1: did not find mandatory property :priority. Mandatory properties are :priority")

  (test-error
    (ipnat/ensure "error-test-2" :priority 0)
    "In ipnat/ensure error-test-2: need exactly one of :content or :from"))
