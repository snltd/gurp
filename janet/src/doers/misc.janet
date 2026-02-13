(use ./lib)
(use ../user-helpers)
(import ../collector)

(def doer :misc)
(def description "A collection of things too small to deserve their own doer.")
(def name-is nil)
(def mandatory-props-ensure {})
(def optional-props-ensure
  {:enable-smb {:types [:string]
                :help "Enable SMB sharing for this username"}
   :nfs-domain {:types [:string]
                :help "NFS domain name"}
   :scheduler {:types [:string]
               :help "The scheduler class to set via dispamdin"}})
(def defaults-ensure {})

(defn ensure
  "Sets miscellaneous system properties"
  [& spec]
  (def name (labelise spec))
  (collector/push :ensure doer (make-ensure-resource)))

(def notes
  ["The misc doer is a placeholder for what Gurp considers \"OS primitives\"
    but which are not big or complex enough to warrant their own doer"])
