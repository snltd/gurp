(use judge)
(use ../lib/gurp)

(deftest "test gem functions"
  (setdyn :role-dyn "test-role")
  (test
    (gem/ensure "wavefront-cli")
    {:gem {:_id "/test-role/gem/wavefront-cli"
           :action :ensure
           :name "wavefront-cli"
           :role "test-role"}})

  (test-error
    (gem/ensure "wavefront-sdk"
                :version 11)
    "Failed to validate user input for gem 'wavefront-sdk' : gem 'wavefront-sdk' has unrecognised key(s): version")

  (test
    (gem/remove "webscale")
    {:gem {:_id "/test-role/gem/webscale"
           :action :remove
           :name "webscale"
           :role "test-role"}}))
