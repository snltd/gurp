(use judge)
(use ../lib/gurp)

(deftest "ensure-resources"
  (setdyn :role-dyn (string "test-role"))

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
