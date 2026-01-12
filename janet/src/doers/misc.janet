(use ./lib)
(import ../collector)

(def doer :misc)
(def description "A collection of things too small to deserve their own doer.")
(def name-is "Package name")
(def mandatory-ensure-props {})
(def optional-ensure-props
  {:enable-smb {:types [:string]
                :help "Enable SMB sharing for this username"}
   :nfs-domain {:types [:string]
                :help "NFS domain name"}
   :scheduler {:types [:string]
               :help "The scheduler class to set via dispamdin"}})
(def default-ensure-prop-values {})

(defn ensure
  "Sets miscellaneous system properties"
  [& spec]
  (def name (labelise spec))
  (collector/push :ensure doer (make-ensure-resource)))
