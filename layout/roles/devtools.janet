(use ../lib/helpers)

(role role
      :packages [(ensure "git")
                 (ensure "rg" :version "latest")
                 (remove "go")]
      :users [(ensure "rob"
                      :uid 264
                      :gid 14
                      :dir "/home/rob")]
      :files [(ensure "sample"
                      :path "/tmp/merp/merp.txt"
                      :source "templates/merp.jinja"
                      :vars {:var-1 "string 1"
                             :var-2 :user/rob/name})]
      :directories [(ensure "merp"
                            :path "/tmp/merp"
                            :owner :user/rob/uid
                            :group :user/rob/group
                            :mode "0755")
                    (ensure "gajerp"
                            :path "/tmp/gajerp"
                            :owner :dir/merp/owner
                            :group "root"
                            :mode "0775")])
