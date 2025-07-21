(use judge)
(use ../lib/gurp)

(deftest "loop-test"
  (test
          ((role test-role
                (pkg/ensure "helix")
                (pkg/ensure "rust")))
    @[{:pkg {:_id "/test-role/pkg/helix"
             :action :ensure
             :name "helix"
             :role "test-role"}}
      {:pkg {:_id "/test-role/pkg/rust"
             :action :ensure
             :name "rust"
             :role "test-role"}}])

  (test
          ((role test-role
                (loop [pkg :in ["helix" "rust"]] (add (pkg/ensure pkg)))))
    @[{:pkg {:_id "/test-role/pkg/helix"
             :action :ensure
             :name "helix"
             :role "test-role"}}
      {:pkg {:_id "/test-role/pkg/rust"
             :action :ensure
             :name "rust"
             :role "test-role"}}
      nil]))

