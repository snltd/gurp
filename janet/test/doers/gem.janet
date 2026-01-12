(use judge)
(use ../../src/collector)
(import ../../src/doers/gem)

(deftest "gem-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (gem/ensure "wavefront-cli" :version "8.0.1")
  (gem/ensure "my-gem" :source "https://my-gem-repo.com")
  (gem/remove "webscale")

  (test *collector*
    @{:ensure @{:gem @[@{:_id "/test-role/gem/wavefront-cli"
                         :name "wavefront-cli"
                         :role "test-role"
                         :version "8.0.1"}
                       @{:_id "/test-role/gem/my-gem"
                         :name "my-gem"
                         :role "test-role"
                         :source "https://my-gem-repo.com"}]}
      :remove @{:gem @[@{:_id "/test-role/gem/webscale"
                         :name "webscale"
                         :role "test-role"}]}}))

(deftest "gem-error"
  (test-error
    (gem/ensure "wavefront-sdk"
                :merp 11)
    "unexpected property 'merp'. Valid properties are: gem-path, version, source, label"))
