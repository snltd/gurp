(use judge)
(use ../lib/gurp)
(use ../lib/encode)

(role devtools
      (pkg/ensure "ooce/developer/rust")
      (pkg/ensure "ooce/developer/git")
      (file/ensure "/tmp/hx-config.txt"
                   :owner :/devtools/file/git-config/owner
                   :source "hx-config")
      (file/ensure "/tmp/git-config.txt"
                   :label "git-config"
                   :owner :/basenode/directory/merp/owner
                   :source "git-config"))

(role basenode
      (section packages
               (pkg/ensure "ooce/editor/helix")
               (pkg/ensure "shell/zsh")
               (pkg/ensure "network/netcat")
               (pkg/ensure "network/openssh")
               (pkg/ensure "network/openssh-server")
               (pkg/ensure "network/rsync")
               (pkg/remove "go"))
      (file/ensure "/tmp/basenode.txt"
                   :content "some words")
      (directory/ensure "/tmp/merp"
                        :label "merp"
                        :owner "rob"
                        :group "sysadmin"
                        :mode "0755")
      (directory/remove "/tmp/junk"))

(host "end-to-end"
      (basenode)
      (devtools))

(deftest "produce-config-struct-for-rust"
  (test (machine-config)
        {:metadata {:name "end-to-end"}
         :resources {:ensure {:directory @[{:_id "/basenode/directory/merp"
                                            :action :ensure
                                            :group "sysadmin"
                                            :label "merp"
                                            :mode "0755"
                                            :name "/tmp/merp"
                                            :owner "rob"
                                            :role "basenode"}]
                              :file @[{:_id "/basenode/file/_tmp_basenode.txt"
                                       :action :ensure
                                       :content "some words"
                                       :group "root"
                                       :mode "0644"
                                       :name "/tmp/basenode.txt"
                                       :owner "root"
                                       :role "basenode"}
                                      {:_id "/devtools/file/_tmp_hx-config.txt"
                                       :action :ensure
                                       :group "root"
                                       :mode "0644"
                                       :name "/tmp/hx-config.txt"
                                       :owner "rob"
                                       :role "devtools"
                                       :source "hx-config"}
                                      {:_id "/devtools/file/git-config"
                                       :action :ensure
                                       :group "root"
                                       :label "git-config"
                                       :mode "0644"
                                       :name "/tmp/git-config.txt"
                                       :owner "rob"
                                       :role "devtools"
                                       :source "git-config"}]
                              :pkg @[{:_id "/basenode/pkg/ooce_editor_helix"
                                      :action :ensure
                                      :name "ooce/editor/helix"
                                      :role "basenode"}
                                     {:_id "/basenode/pkg/shell_zsh"
                                      :action :ensure
                                      :name "shell/zsh"
                                      :role "basenode"}
                                     {:_id "/basenode/pkg/network_netcat"
                                      :action :ensure
                                      :name "network/netcat"
                                      :role "basenode"}
                                     {:_id "/basenode/pkg/network_openssh"
                                      :action :ensure
                                      :name "network/openssh"
                                      :role "basenode"}
                                     {:_id "/basenode/pkg/network_openssh-server"
                                      :action :ensure
                                      :name "network/openssh-server"
                                      :role "basenode"}
                                     {:_id "/basenode/pkg/network_rsync"
                                      :action :ensure
                                      :name "network/rsync"
                                      :role "basenode"}
                                     {:_id "/devtools/pkg/ooce_developer_rust"
                                      :action :ensure
                                      :name "ooce/developer/rust"
                                      :role "devtools"}
                                     {:_id "/devtools/pkg/ooce_developer_git"
                                      :action :ensure
                                      :name "ooce/developer/git"
                                      :role "devtools"}]}
                     :remove {:directory @[{:_id "/basenode/directory/_tmp_junk"
                                            :action :remove
                                            :group "root"
                                            :mode "0755"
                                            :name "/tmp/junk"
                                            :owner "root"
                                            :role "basenode"}]
                              :pkg @[{:_id "/basenode/pkg/go"
                                      :action :remove
                                      :name "go"
                                      :role "basenode"}]}}}))
