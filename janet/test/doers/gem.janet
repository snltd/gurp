(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/gem)

(deftest gem
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "gem" (curenv))

  (test *collector*
    @{:ensure @{:gem @[{:_id "/test-role/gem/wavefront-cli"
                        :name "wavefront-cli"
                        :role "test-role"}
                       {:_id "/test-role/gem/my-gem"
                        :gem-path "/opt/pkgin/bin/gem"
                        :name "my-gem"
                        :role "test-role"
                        :source "https://my-gem-repo.com"
                        :version "1.2.3"}]}
      :remove @{:gem @[{:_id "/test-role/gem/webscale"
                        :name "webscale"
                        :role "test-role"}]}}))

(deftest gem-error
  (test-error
    (gem/ensure "wavefront-sdk"
                :merp 11)
    "unexpected property :merp. Valid properties are :gem-path, :version, :source, :label"))
