(use judge)
(use ../../src/collector)
(import ../../src/doers/ipfilter)

(deftest "ipfilter-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (ipfilter/remove "empty-test")

  (ipfilter/ensure "test-1"
                   :from "test/ipfilter-test"
                   :priority 2)

  (ipfilter/ensure "test-2"
                   :priority 1
                   :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\n
rdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin")

  (test *collector*
        @{:ensure @{:ipfilter @[{:_id "/test-role/ipfilter/test-1"
                                 :from "test/ipfilter-test"
                                 :name "test-1"
                                 :priority 2
                                 :role "test-role"}
                                {:_id "/test-role/ipfilter/test-2"
                                 :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\nrdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin"
                                 :name "test-2"
                                 :priority 1
                                 :role "test-role"}]}
          :remove @{:ipfilter @[{:_id "/test-role/ipfilter/empty-test"
                                 :name "empty-test"
                                 :role "test-role"}]}}))

(deftest "ipfilter-errors"
  (test-error
    (ipfilter/ensure "error-test-1" :from "test/ipfilter")
    "did not find mandatory property :priority. Mandatory properties are :priority")

  (test-error (ipfilter/ensure "error-test-2" :priority 0) "need exactly one of :content or :from"))
