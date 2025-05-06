(import ./lib/helpers)
(import ./roles/devtools)

(helpers/host "example"
  :roles [
    "devtools"
  ])

# (defn machine-config []
#   (helpers/merge-roles
#     (devtools/role)))

(pp (machine-config))

