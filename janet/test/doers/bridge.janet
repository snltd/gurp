(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/bridge)

(deftest bridge
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "bridge" (curenv))
  
  (test *collector*
    @{:ensure @{:bridge @[{:_id "/test-role/bridge/basic"
                           :force-protocol 3
                           :forward-delay 15
                           :hello-time 2
                           :max-age 20
                           :name "basic"
                           :priority 32768
                           :protect "stp"
                           :role "test-role"}
                          {:_id "/test-role/bridge/with_links"
                           :force-protocol 3
                           :forward-delay 15
                           :hello-time 2
                           :links ["stub0" "vnic0" "e1000g0"]
                           :max-age 30
                           :name "with_links"
                           :priority 4096
                           :protect "stp"
                           :role "test-role"}]}
      :remove @{:bridge @[{:_id "/test-role/bridge/unwanted"
                           :name "unwanted"
                           :role "test-role"}]}}))

(deftest bridge-errors
  (test-error
    (bridge/ensure "test_d" :oops "wat?")
    "unexpected property :oops. Valid properties are :priority, :links, :label, :max-age, :force-protocol, :protect, :forward-delay, :hello-time")
  (test-error
    (bridge/ensure "test_e" :priority "high!")
    "priority is of type :string. Allowed types :number"))
