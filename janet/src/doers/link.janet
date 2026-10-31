(use ./definers)
(import ../collector)

(defdoer :link
  "Create and remove links."
  :name-is "Qualified path to the link that will be created"
  (ensure
    (property "source" [:string] :mandatory "The file to which we will link")
    (property "type" [:string] :mandatory "The type of link"
              :values ["symbolic" "hard"]
              :default "symbolic")
    (property "force-link" [:boolean] :optional
              "If the link target already exists and this flag is `true`, Gurp
              will remove it and replace it with a link. If `false`, a
              pre-existing target causes an error"))

  (notes
    "If the source doesn't exist, you get an error."
    "Files and directories are ensured before links, so you can link
    Gurp-managed resources."))

(make-fn :ensure :link)
(make-fn :remove :link)
