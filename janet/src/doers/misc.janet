(use ./lib)
(use ../dsl)
(import ../collector)

(defdoer :misc
  "A collection of things too small to deserve their own doer."
  :name-is nil

  :optional-props-ensure
  {:enable-smb {:types [:string]
                :help "Enable SMB sharing for this username"}
   :nfs-domain {:types [:string]
                :help "NFS domain name"}
   :scheduler {:types [:string]
               :help "The scheduler class to set via dispamdin"}}

  :notes
  ["The misc doer is a placeholder for what Gurp considers \"OS primitives\"
    but which are not big or complex enough to warrant their own doer"])

(defn ensure
  "Sets miscellaneous system properties"
  [& spec]
  (let [name (labelise spec)]
    (collector/push :ensure doer (make-ensure-resource))))
