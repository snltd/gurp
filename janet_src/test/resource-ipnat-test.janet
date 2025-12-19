(use judge)
(use ../lib/gurp)

(deftest "ipnat-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (ipnat/remove "empty-test")

  (ipnat/ensure "test-1"
                :from "test/ipnat-test"
                :flags [:disable-resolution]
                :in-zone "test-zone")

  (ipnat/ensure "test-2"
                :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robin\n
rdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin")

  (ipnat/ensure "test-3"
                :flags [:disable-resolution]
                :global-zone "test-zone")

  (test *collector*
    @{:ensure @{:ipnat @[{:_id "/test-role/ipnat/test-1"
                          :flags [:disable-resolution]
                          :from "test/ipnat-test"
                          :in-zone "test-zone"
                          :name "test-1"
                          :role "test-role"}
                         {:_id "/test-role/ipnat/test-2"
                          :content "rdr le0 203.1.2.3/32 port 80 -> 203.1.2.3,203.1.2.4 port 80 tcp round-robinrdr le0 203.1.2.3/32 port 80 -> 203.1.2.5 port 80 tcp round-robin"
                          :name "test-2"
                          :role "test-role"}
                         {:_id "/test-role/ipnat/test-3"
                          :flags [:disable-resolution]
                          :global-zone "test-zone"
                          :name "test-3"
                          :role "test-role"}]}
      :remove @{:ipnat @[{:_id "/test-role/ipnat/empty-test"
                          :name "empty-test"
                          :role "test-role"}]}}))
