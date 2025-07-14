(use judge)
(use ../lib/gurp)

(deftest "test svc functions"
  (test
          (svc/ensure "important/service"
                      :state "enabled"
                      :restarted-by [:test-role/file/stub])
    {:svc {:_id "/NO-ROLE/svc/important_service"
           :action :ensure
           :name "important/service"
           :reloaded-by []
           :restarted-by [:test-role/file/stub]
           :state "enabled"}})

  (test-error
    (svc/ensure "too/many/keys"
                :state "enabled"
                :volume 11
                :strings: 12)
    "svc 'too/many/keys' has unrecognised key(s): strings:, volume")

  (test-error
    (svc/ensure "what/should/i/do")
    "svc missing required key(s): state"))
