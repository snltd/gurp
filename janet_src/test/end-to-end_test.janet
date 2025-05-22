(use judge)

(import ./resources/example)
(import ./resources/roles/basenode)
(import ./resources/roles/devtools)
(use ../lib/gurp)

(deftest "produce-config-struct-for-rust"
  (test (example/machine-config)
    {:metadata {:name "example"}
     :resources {:ensure {:directory @[{:_id "/basenode/directory/merp"
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
                                      :role "basenode"}]}}}))
