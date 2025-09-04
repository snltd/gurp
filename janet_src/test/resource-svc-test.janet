(use judge)
(use ../lib/gurp)

(deftest "svc-resources"
  (set *collector* (new-collector))

  (svc/ensure "important/service"
              :state "enabled"
              :restarted-by [:/test-role/file/stub])

  (test *collector*
    @{:ensure @{:svc @[{:_id "/NO-ROLE/svc/important_service"
                        :name "important/service"
                        :reloaded-by @[]
                        :restarted-by @["/test-role/file/stub"]
                        :state "enabled"}]}
      :remove @{}}))

(deftest "svc-error"
  (test-error
    (svc/ensure "too/many/keys"
                :state "enabled"
                :volume 11
                :strings: 12)
    "Failed to validate user input for svc 'too/many/keys': svc 'too/many/keys' has unrecognised key(s): strings:, volume")

  (test-error
    (svc/ensure "what/should/i/do")
    "Failed to validate user input for svc 'what/should/i/do': svc missing required key(s): state"))
