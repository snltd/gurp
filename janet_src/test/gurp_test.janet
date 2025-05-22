(use judge)
(use ../lib/gurp)

(test-macro
  (host "example-node" (role "basenode") (role "devtools"))
  (defn machine-config
    []
    {:metadata {:name "example-node"} :resources (group-by-action-and-type (flatten (tuple (role "basenode") (role "devtools"))))}))

(test-macro
  (role basenode
        (package/ensure "helix")
        (package/remove "go")
        (file/ensure "basenode_file"
                     :path "/tmp/basenode.txt"
                     :content "some words")
        (directory/ensure "merp"
                          :path "/tmp/merp"
                          :owner :user/rob/uid
                          :group :user/rob/group
                          :mode "0755")
        (directory/remove "junk"
                          :path "/tmp/junk"))
  (defn basenode
    []
    (setdyn :role-dyn (string (quote basenode)))
    (collect-resources (package/ensure "helix") (package/remove "go") (file/ensure "basenode_file" :path "/tmp/basenode.txt" :content "some words") (directory/ensure "merp" :path "/tmp/merp" :owner :user/rob/uid :group :user/rob/group :mode "0755") (directory/remove "junk" :path "/tmp/junk"))))

(deftest "remove-package-resource"
  (test (package/remove "helix")
        {:package {:_id "//package/helix"
                   :action :remove
                   :name "helix"}}))

(deftest "ensure-resources"
  (setdyn :role-dyn (string "test-role"))

  (test "gibbus"
        "gibbus")
  (test (file/ensure "my-file"
                     :path "/my/file"
                     :owner "rob")
        {:file {:_id "/test-role/file/my-file"
                :action :ensure
                :group "root"
                :name "my-file"
                :owner "rob"
                :path "/my/file"
                :role "test-role"}})

  (test (directory/ensure "deep-dir"
                          :path "/my/deep/nested/directory/needs/recursion"
                          :recurse true
                          :owner "daemon")
        {:directory {:_id "/test-role/directory/deep-dir"
                     :action :ensure
                     :group "root"
                     :name "deep-dir"
                     :owner "daemon"
                     :path "/my/deep/nested/directory/needs/recursion"
                     :recurse true
                     :role "test-role"}})

  (setdyn :role-dyn nil))

(deftest "test-group-by-action"
  (def data
    @[{:package {:_id "/basenode/package/helix" :action :ensure :name "helix" :role "basenode"}} {:package {:_id "/basenode/package/go" :action :remove :name "go" :role "basenode"}} {:file {:_id "/basenode/file/basenode_file" :action :ensure :content "some words" :group "root" :name "basenode_file" :owner "root" :path "/tmp/basenode.txt" :role "basenode"}} {:directory {:_id "/basenode/directory/merp" :action :ensure :group :user/rob/group :mode "0755" :name "merp" :owner :user/rob/uid :path "/tmp/merp" :recurse false :role "basenode"}} {:directory {:_id "/basenode/directory/junk" :action :remove :group "root" :name "junk" :owner "root" :path "/tmp/junk" :recurse false :role "basenode"}} {:package {:_id "/devtools/package/rust" :action :ensure :name "rust" :role "devtools"}} {:package {:_id "/devtools/package/git" :action :ensure :name "git" :role "devtools"}} {:file {:_id "/devtools/file/git_config" :action :ensure :group "root" :name "git_config" :owner "root" :path "/tmp/git-config.txt" :role "devtools" :source "git-config"}}])
  (test (group-by-action-and-type data)
    {:ensure {:directory @[{:_id "/basenode/directory/merp"
                            :action :ensure
                            :group :user/rob/group
                            :mode "0755"
                            :name "merp"
                            :owner :user/rob/uid
                            :path "/tmp/merp"
                            :recurse false
                            :role "basenode"}]
              :file @[{:_id "/basenode/file/basenode_file"
                       :action :ensure
                       :content "some words"
                       :group "root"
                       :name "basenode_file"
                       :owner "root"
                       :path "/tmp/basenode.txt"
                       :role "basenode"}
                      {:_id "/devtools/file/git_config"
                       :action :ensure
                       :group "root"
                       :name "git_config"
                       :owner "root"
                       :path "/tmp/git-config.txt"
                       :role "devtools"
                       :source "git-config"}]
              :package @[{:_id "/basenode/package/helix"
                          :action :ensure
                          :name "helix"
                          :role "basenode"}
                         {:_id "/devtools/package/rust"
                          :action :ensure
                          :name "rust"
                          :role "devtools"}
                         {:_id "/devtools/package/git"
                          :action :ensure
                          :name "git"
                          :role "devtools"}]}
     :remove {:directory @[{:_id "/basenode/directory/junk"
                            :action :remove
                            :group "root"
                            :name "junk"
                            :owner "root"
                            :path "/tmp/junk"
                            :recurse false
                            :role "basenode"}]
              :package @[{:_id "/basenode/package/go"
                          :action :remove
                          :name "go"
                          :role "basenode"}]}}))
