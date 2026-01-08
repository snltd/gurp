(use judge)
(use ../lib/gurp)

(set *collector* (new-collector))

(role role-a
      (file/ensure "/tmp/a1" :content "blah")
      (pkg/ensure "ruby")
      (loop [pkg :in ["python" "perl"]]
            (pkg/remove pkg))
      (pkg/ensure "rust"))

(role role-b
      (file/ensure "/tmp/b1" :content "blah-blah")
      (file/remove "/bad/file")
      (pkg/ensure "helix")
      (pkg/remove "go"))

(host "test-host"
      (role-a)
      (role-b))

(deftest "test-collector"
  (machine-config)
  (test *collector*
    @{:ensure @{:file @[{:_id "/role-a/file/_tmp_a1"
                         :content "blah"
                         :group "root"
                         :mode "0644"
                         :name "/tmp/a1"
                         :owner "root"
                         :role "role-a"}
                        {:_id "/role-b/file/_tmp_b1"
                         :content "blah-blah"
                         :group "root"
                         :mode "0644"
                         :name "/tmp/b1"
                         :owner "root"
                         :role "role-b"}]
                :pkg @[{:_id "/role-a/pkg/ruby"
                        :name "ruby"
                        :role "role-a"}
                       {:_id "/role-a/pkg/rust"
                        :name "rust"
                        :role "role-a"}
                       {:_id "/role-b/pkg/helix"
                        :name "helix"
                        :role "role-b"}]}
      :remove @{:file @[{:_id "/role-b/file/_bad_file"
                         :name "/bad/file"
                         :role "role-b"}]
                :pkg @[{:_id "/role-a/pkg/python"
                        :name "python"
                        :role "role-a"}
                       {:_id "/role-a/pkg/perl"
                        :name "perl"
                        :role "role-a"}
                       {:_id "/role-b/pkg/go"
                        :name "go"
                        :role "role-b"}]}}))

