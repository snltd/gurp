(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/svc)

(deftest svc
  (set *collector* (new-collector))

  (import-tests "svc" (curenv))

  (test *collector*
        @{:ensure @{:svc @[{:_id "/NO-ROLE/svc/important_service"
                            :name "important/service"
                            :reloaded-by @[]
                            :restarted-by @["/test-role/file/stub"]
                            :role "NO-ROLE"
                            :state "enabled"}]}
          :remove @{}})

  (test-error
    (svc/ensure "too/many/keys"
                :state "enabled"
                :volume 11
                :strings: 12)
    "In svc/ensure too/many/keys: unexpected property :strings:. Valid properties are :state, :restarted-by, :reloaded-by, :label")

  (test-error
    (svc/ensure "what/should/i/do")
    "In svc/ensure what/should/i/do: did not find mandatory property :state. Mandatory properties are :state"))
