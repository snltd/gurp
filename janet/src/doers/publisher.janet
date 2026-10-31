(use ./definers)
(import ../collector)
(import ./publisher/origin :prefix "" :export true)
(import ./publisher/mirror :prefix "" :export true)

# publisher
#   ├── sticky (bool)
#   ├── enabled (bool)
#   ├── search-first / search-before / search-after (ordering)
#   ├── ssl-key, ssl-cert
#   └── origins[]
#         ├── uri
#         └── proxy (optional, per-URI)
#       mirrors[]
#         ├── uri
#         └── proxy (optional, per-URI) 

(defdoer :publisher
  "Add and remove pkg(5) publisher origins."
  :name-is "Publisher name"
  (ensure
    :expand-resources {:origin false :mirror false}
    (property "origin" [:tuple] :mandatory
              "List of origins, created with publisher/origin ")
    (property "mirror" [:tuple] :optional
              "List of mirrors, created with publisher/mirror")
    (property "search-first" [:boolean] :optional
              "Search this publisher first")))

(make-fn :ensure :publisher)
(make-fn :remove :publisher)
