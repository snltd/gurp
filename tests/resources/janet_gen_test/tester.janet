(import roles/devtools)
(import roles/basenode)

(host "example"
           :roles ["devtools"
                   "basenode"])

(run-machine-configuration (machine-config))
