(def verbose true)

# TODO make it so you can override this somewhere
(def dir-defaults @{:user "root" :group "root" :mode "755"})

(def dry-run false)
(def debug true)

(defn say [& msg]
  (print (string ;msg)))

(defn debug [& msg]
  (when (true? debug) (print (string ;msg))))
  
# (defn directory [& dir]
#   (say "ensuring directory" (:path dir))
# )

(defn directory-now 
  "Returns the current state of the given directory. Nil if it does not exist"
  [path]
  (let [stat (os/stat path)]
    (if stat
      {:group (get stat :gid)
       :user (get stat :uid)
       :mode (get stat :int-permissions)
      }
      nil)))

(defn enact [anon-fn]
  (when (false? dry-run) (anon-fn)))

# In a directory we can modify user, group, and mode
(defn directory-new [path]
  (say "creating directory: " path)
  (enact |(os/mkdir path)))

  #" [user=" (opts :user) 
  #" group=" (opts :group)
  #" mode=" (os/perm-string (opts :mode))
  #"]")) 

(defn directory-todo
  "Returns the diff of the current and desired states"
  [current desired] )

(defn correct-directory [opts])

(defn directory-check [path now want]
  # loop over want and compare to now
  # (pp (type want))
  # (pp (keys want))
  # (pp want)
  (eachp setting want (do
    (let [[k v] setting]
      (print k )
      )
    ))
  # (pp now)
)

(defn directory [path & body] 
  (if (os/stat path) 
    (debug "directory exists: " path)
    (directory-new path))
  (let [now (directory-now path)
        want (table ;body)]
    (table/setproto want dir-defaults)
    (when (want :mode) 
      (put want :mode (scan-number (want :mode) 8)))
    (directory-check path now want))) 
