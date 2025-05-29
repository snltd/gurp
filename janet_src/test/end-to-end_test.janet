(use judge)
(use ../lib/gurp)

(role devtools
      (pkg/ensure "rust")
      (pkg/ensure "git")
      (file/ensure "hx_config"
                   :owner :/devtools/file/git_config/owner
                   :path "/tmp/hx-config.txt"
                   :source "hx-config")
      (file/ensure "git_config"
                   :owner :/basenode/directory/merp/owner
                   :path "/tmp/git-config.txt"
                   :source "git-config"))

(role basenode
      (pkg/ensure "helix")
      (pkg/remove "go")
      (file/ensure "basenode_file"
                   :path "/tmp/basenode.txt"
                   :content "some words")
      (directory/ensure "merp"
                        :path "/tmp/merp"
                        :owner "rob"
                        :group "sysadmin"
                        :mode "0755")
      (directory/remove "junk"
                        :path "/tmp/junk"))

(host "end-to-end"
      (basenode)
      (devtools))

(deftest "produce-config-struct-for-rust"
  (test (machine-config)
        {:metadata {:name "end-to-end"}
         :resources {:ensure {:directory @[{:_id "/basenode/directory/merp"
                                            :action :ensure
                                            :group "sysadmin"
                                            :mode "0755"
                                            :name "merp"
                                            :owner "rob"
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
                                      {:_id "/devtools/file/hx_config"
                                       :action :ensure
                                       :group "root"
                                       :name "hx_config"
                                       :owner "rob"
                                       :path "/tmp/hx-config.txt"
                                       :role "devtools"
                                       :source "hx-config"}
                                      {:_id "/devtools/file/git_config"
                                       :action :ensure
                                       :group "root"
                                       :name "git_config"
                                       :owner "rob"
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
                                          :role "basenode"}]}}}))
