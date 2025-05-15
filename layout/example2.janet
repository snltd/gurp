(import ./lib/helpers)
(import ./roles/devtools)
(import ./roles/basenode)

# (pp (macex1
# '(helpers/host "example"
#   :roles [
#     "devtools"
#   ])

# ))

# (defn machine-config []
#   (helpers/merge-roles
#     (devtools/role)))

(helpers/host "example"
  :roles [
    "basenode"
    "devtools"
  ])
 
(pp (machine-config))


