(use ./lib)
(import ../collector)

(defdoer :link
  "Create and remove links."
  :name-is "Qualified path to the link that will be created"

  :mandatory-props-ensure
  {:source {:types [:string]
            :help "The file to which we will link"}
   :force-link {:types [:boolean]
                :help "If the link target already exists and this flag is true,
                       Gurp will remove it and replace it with a link. If false,
                      a pre-existing target causes an error"}
   :type {:types [:string]
          :help "The type of link: symbolic or hard"}}

  :defaults-ensure
  {:type "symbolic"
   :force-link false}

  :notes
  ["If the source doesn't exist, you get an error."
   "Files and directories are ensured before links, so you can link
    Gurp-managed resources."])

(defensure "link")
(defremove "link")
