(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/ipnat)

(deftest ipnat
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "ipnat" (curenv))

  (test *collector*
        @{:ensure @{:ipnat @[{:_id "/test-role/ipnat/test-1"
                              :from "test/ipnat-test"
                              :name "test-1"
                              :priority 2
                              :role "test-role"}
                             {:_id "/test-role/ipnat/test-2"
                              :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\nrdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin"
                              :name "test-2"
                              :priority 1
                              :role "test-role"}]}
          :remove @{:ipnat @[{:_id "/test-role/ipnat/empty-test"
                              :name "empty-test"
                              :role "test-role"}]}}))

(deftest ipnat-errors
  (test-error
    (ipnat/ensure "error-test-1" :from "test/ipnat")
    "did not find mandatory property :priority. Mandatory properties are :priority")

  (test-error (ipnat/ensure "error-test-2" :priority 0) "need exactly one of :content or :from"))
