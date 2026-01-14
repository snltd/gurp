(use ./lib)
(import ../collector)

(def doer :user)
(def description "Manage Unix users")
(def name-is "User's username")
(def mandatory-props-ensure
  {:gecos {:types [:string]
           :help "User's name or description"}
   :home-dir {:types [:string]
              :help "User's home dir"}
   :primary-group {:types [:string :number]
                   :help "Group name or GID to which user belongs"}
   :shell {:types [:string]
           :help "User's shell"}
   :uid {:types [:number]
         :help "UID of user"}})
(def optional-props-ensure
  {:other-groups {:types [:tuple]
                  :help "Group names (:string) or GIDs (:number) to which user belongs"}
   :password-hash {:types [:string]
                   :help "Hash to insert in /etc/shadow"}
   :profiles {:types [:tuple]
              :help "List of existing profiles (:string)"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:shell "/bin/zsh"
   :primary-group "staff"})
(def defaults-remove {})

(defn ensure
  "Given an apk package name, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given an apk package name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
