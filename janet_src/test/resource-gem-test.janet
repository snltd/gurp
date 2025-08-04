(use judge)
(use ../lib/gurp)

(deftest "gem-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (gem/ensure "wavefront-cli")
  (gem/remove "webscale")

  (test *collector*
        @{:ensure @{:gem @[{:_id "/test-role/gem/wavefront-cli"
                            :name "wavefront-cli"
                            :role "test-role"}]}
          :remove @{:gem @[{:_id "/test-role/gem/webscale"
                            :name "webscale"
                            :role "test-role"}]}}))

(deftest "gem-error"
  (test-error
    (gem/ensure "wavefront-sdk"
                :version 11)
    "Failed to validate user input for gem 'wavefront-sdk' : gem 'wavefront-sdk' has unrecognised key(s): version"))
