(import roles/devtools)
(import roles/basenode)

(host "example"
           :roles ["devtools"
                   "basenode"])



(pp (machine-config))
# (run-machine-configuration (machine-config))
