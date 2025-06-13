(use judge)
(use ../lib/gurp)

(test-macro
  (host "example-node" (role "basenode") (role "devtools"))
  (defn machine-config
    []
    {:metadata {:name "example-node"} :resources (group-by-action-and-type (flatten (tuple (role "basenode") (role "devtools"))))}))

(test-macro
  (role basenode
        (pkg/ensure "helix")
        (pkg/remove "go")
        (file/ensure "/tmp/basenode.txt"
                     :content "some words")
        (directory/ensure "/tmp/merp"
                          :owner "tester"
                          :group "tester"
                          :mode "0755")
        (directory/remove "/tmp/junk"))
  (defn basenode
    []
    (setdyn :role-dyn (string (quote basenode)))
    (collect-resources (pkg/ensure "helix") (pkg/remove "go") (file/ensure "/tmp/basenode.txt" :content "some words") (directory/ensure "/tmp/merp" :owner "tester" :group "tester" :mode "0755") (directory/remove "/tmp/junk"))))

(deftest "remove-pkg-resource"
  (test (pkg/remove "/ooce/editor/helix")
        {:pkg {:_id "/NO-ROLE/pkg/_ooce_editor_helix"
               :action :remove
               :name "/ooce/editor/helix"}}))

(deftest "ensure-resources"
  (setdyn :role-dyn (string "test-role"))

  (test "gibbus"
        "gibbus")
  (test (file/ensure "/my/file"
                     :owner "rob")
    {:file {:_id "/test-role/file/_my_file"
            :action :ensure
            :group "root"
            :mode "0644"
            :name "/my/file"
            :owner "rob"
            :role "test-role"}})

  (test (directory/ensure "/my/deep/nested/directory/needs/recursion"
                          :label "deep-dir"
                          :owner "daemon")
    {:directory {:_id "/test-role/directory/deep-dir"
                 :action :ensure
                 :group "root"
                 :label "deep-dir"
                 :mode "0755"
                 :name "/my/deep/nested/directory/needs/recursion"
                 :owner "daemon"
                 :role "test-role"}})

  (setdyn :role-dyn nil))

(deftest "test-group-by-action"
  (def data
    @[{:pkg {:_id "/basenode/pkg/helix" :action :ensure :name "helix" :role "basenode"}} {:pkg {:_id "/basenode/pkg/go" :action :remove :name "go" :role "basenode"}} {:file {:_id "/basenode/file/basenode_file" :action :ensure :content "some words" :group "root" :name "basenode_file" :owner "root" :path "/tmp/basenode.txt" :role "basenode"}} {:directory {:_id "/basenode/directory/merp" :action :ensure :group "tester" :mode "0755" :name "merp" :owner "tester" :path "/tmp/merp" :role "basenode"}} {:directory {:_id "/basenode/directory/junk" :action :remove :group "root" :name "junk" :owner "root" :path "/tmp/junk" :role "basenode"}} {:pkg {:_id "/devtools/pkg/rust" :action :ensure :name "rust" :role "devtools"}} {:pkg {:_id "/devtools/pkg/git" :action :ensure :name "git" :role "devtools"}} {:file {:_id "/devtools/file/git_config" :action :ensure :group "root" :name "git_config" :owner "root" :path "/tmp/git-config.txt" :role "devtools" :source "git-config"}}])
  (test (group-by-action-and-type data)
        {:ensure {:directory @[{:_id "/basenode/directory/merp"
                                :action :ensure
                                :group "tester"
                                :mode "0755"
                                :name "merp"
                                :owner "tester"
                                :path "/tmp/merp"
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
                  :pkg @[{:_id "/basenode/pkg/helix"
                          :action :ensure
                          :name "helix"
                          :role "basenode"}
                         {:_id "/devtools/pkg/rust"
                          :action :ensure
                          :name "rust"
                          :role "devtools"}
                         {:_id "/devtools/pkg/git"
                          :action :ensure
                          :name "git"
                          :role "devtools"}]}
         :remove {:directory @[{:_id "/basenode/directory/junk"
                                :action :remove
                                :group "root"
                                :name "junk"
                                :owner "root"
                                :path "/tmp/junk"
                                :role "basenode"}]
                  :pkg @[{:_id "/basenode/pkg/go"
                          :action :remove
                          :name "go"
                          :role "basenode"}]}}))

(deftest "test-this"
  (setdyn :role-dyn (string (quote basenode)))
  (test (this "file" "the-label" "owner") "/basenode/file/the-label/owner")
  (test (this "file" "the-label") "/basenode/file/the-label")
  (setdyn :role-dyn nil))
  
(test-macro 
  (section "test-section"
    (svc/ensure "cron")
    (directory/ensure "/tmp/test"))
  (do
    (svc/ensure "cron")
    (directory/ensure "/tmp/test")))
