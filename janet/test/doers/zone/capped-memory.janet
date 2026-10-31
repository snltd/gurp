(use judge)
(use ../test-lib)
(import ../../../src/doers/zone)

(deftest zone/capped-memory
  (test
    (import-test "zone/capped-memory-01.janet")
    {:capped-memory @{:physical "2G" :swap "4G"}})

  (test
    (zone/capped-memory :physical "2G")
    {:capped-memory @{:physical "2G"}})

  (test-error
    (zone/capped-memory)
    "In zone/capped-memory NO-NAME: need at-least one of :physical, :swap, :locked")

  (test-error
    (zone/capped-memory :swap 1000)
    "In zone/capped-memory NO-NAME: swap is of type :number. Allowed types :string")

  (test-error
    (zone/capped-memory
      :physical "2G"
      :virtual "10G")
    "In zone/capped-memory NO-NAME: unexpected property :virtual. Valid properties are :label, :locked, :physical, :swap"))
