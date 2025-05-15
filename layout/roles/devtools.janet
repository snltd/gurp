(import ../vars)
(use ../lib/helpers)
(use ../defaults)

(role role
      :packages [(ensure "git")
                 (ensure "rg")
                 (remove "go")]
      :files [(ensure "sample"
                      :path "/tmp/merp/merp.txt"
                      :source "templates/merp.jinja")]
      :directories [(ensure "gajerp"
                            :path "/tmp/gajerp"
                            :owner :dir/merp/owner
                            :group "root"
                            :mode "0775")])
