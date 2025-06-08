(use judge)
(use ../lib/gurp)

(role devtools
      (pkg/ensure "rust")
      (pkg/ensure "git")
      (file/ensure "/tmp/hx-config.txt"
                   :owner :/devtools/file/git-config/owner
                   :source "hx-config")
      (file/ensure "/tmp/git-config.txt"
                   :label "git-config"
                   :owner :/basenode/directory/merp/owner
                   :source "git-config"))

(role basenode
      (pkg/ensure "helix")
      (pkg/remove "go")
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
