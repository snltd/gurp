(use judge)
(use ./lib/test-lib)
(use ../../src/collector)
(import ../../src/doers/aggr)

(deftest aggr
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "aggr")

  (test *collector*
    @{:ensure @{"aggr" @[{:_id "/test-role/aggr/aggr0"
                          :lacp-timer "long"
                          :links ["vioif1" "vioif2" "vioif3"]
                          :name "aggr0"
                          :policy "L2,L3"
                          :role "test-role"}]}
      :remove @{"aggr" @[{:_id "/test-role/aggr/aggr1"
                          :name "aggr1"
                          :role "test-role"}]}}))

(deftest aggr-error
  (test-error
    (aggr/ensure "aggr2")
    "In aggr/ensure aggr2: did not find mandatory property :links. Mandatory properties are :links, :policy")

  (test-error
    (aggr/ensure "aggr3"
                          :links ["vioif1" "vioif2" "vioif3"]
                          :policy "L2,L3"
                  :oops "wat?")
    "In aggr/ensure aggr3: unexpected property :oops. Valid properties are :address, :label, :lacp-mode, :lacp-timer, :links, :policy"))

