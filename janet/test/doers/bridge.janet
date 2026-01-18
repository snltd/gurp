(use judge)
(use ../../src/collector)
(import ../../src/doers/bridge)

(deftest bridge
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (bridge/ensure "test_a")

  (bridge/ensure "test_b"
                 :links ["stub0" "vnic0" "e1000g0"]
                 :priority 4096
                 :max-age 30)

  (bridge/remove "test_c")
  
  (test *collector*
    @{:ensure @{:bridge @[{:_id "/test-role/bridge/test_a"
                           :force-protocol 3
                           :forward-delay 15
                           :hello-time 2
                           :max-age 20
                           :name "test_a"
                           :priority 32768
                           :role "test-role"}
                          {:_id "/test-role/bridge/test_b"
                           :force-protocol 3
                           :forward-delay 15
                           :hello-time 2
                           :links ["stub0" "vnic0" "e1000g0"]
                           :max-age 30
                           :name "test_b"
                           :priority 4096
                           :role "test-role"}]}
      :remove @{:bridge @[{:_id "/test-role/bridge/test_c"
                           :name "test_c"
                           :role "test-role"}]}}))

(deftest bridge-errors
  (test-error
    (bridge/ensure "test_d" :oops "wat?")
    "unexpected property :oops. Valid properties are :priority, :links, :label, :max-age, :force-protocol, :protect, :forward-delay, :hello-time")
  (test-error
    (bridge/ensure "test_e" :priority "high!")
    "priority is of type :string. Allowed types :number"))
