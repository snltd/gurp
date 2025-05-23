(use roles/devtools)
(use roles/basenode)

(host "example"
      (basenode)
      (devtools))

(run-machine-configuration (machine-config))
