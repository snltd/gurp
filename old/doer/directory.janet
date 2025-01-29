(use ../lib/core)
(import ../lib/maps)
(use sh)

# You can't recursively create directories. By choice.
# TODO make it so you can override this somewhere
(def -defaults @{
  :user "root" 
  :group "root" 
  :mode "755" })
  #:rmstyle "normal"})  ## TODO where will this go?

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

# In a directory we can modify user, group, and mode
(defn -create [path]
  (say "creating directory: " path)
  (enact! |(os/mkdir path)))

  #" [user=" (opts :user) 
  #" group=" (opts :group)
  #" mode=" (os/perm-string (opts :mode))
  #"]")) 

(defn check 
  [setting now want]
  (say-debug setting " [" (now setting) " vs " (want setting) "]"))

(defn correct-user [path now want]
  (if (= now want)
    (say-debug "user is already " now)
    (do
      (say "changing user from " now " to " want)
      # Janet doesn't have a native chown or chgrp
      (enact! |($ chown ,want ,path)))))

(defn correct-group [path now want]
  (if (= now want)
    (say-debug "group is already " now)
    (do
      (say "changing group from " now " to " want)
      (enact! |($ chgrp ,want ,path)))))

(defn correct-mode [path now want]
  (pp now)
  (pp want)
  (if (= now want)
    (say-debug "mode is already " now)
    (do
      (say (string/format "changing mode from %d to %d" now want))
      (enact! (os/chmod path want)))))

(defn correct [path now want]
  (correct-user path (maps/uid->name (now :user)) (want :user))
  (correct-group path (maps/gid->name (now :group)) (want :user))
  (correct-mode path (now :mode) (scan-number (string/format "%d" (want :mode)))))

(defn must [path & body] 
  (if (os/stat path) 
    (say-debug "directory exists: " path)
    (create path))
  (let [now (directory-now path)
        want (table ;body)]
    (table/setproto want dir-defaults)
    (correct path now want))) 

(defn -must-not [path & body]
  (pp body)
  (when (os/stat path) 
    (say-debug path " exists and shouldn't")
    (case (get (table ;body) :rmstyle)
      "recursive" |($ rmdir -p ,path)
      "nuke" (pp |($ rm -fr ,path))
      nil (fn [] os/rmdir path))
      (error "rmstyle must be recursive or nuke"))))

(defn must-not [path & body]
  (enact! (-must-not path body)))
