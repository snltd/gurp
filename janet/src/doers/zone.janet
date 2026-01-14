(use ./lib)
(use ../lib)
(use ../user-helpers)
(import ./zone/attr :prefix "" :export true)
(import ./zone/bhyve :prefix "" :export true)
(import ./zone/bootstrap :prefix "" :export true)
(import ./zone/fs :prefix "" :export true)
(import ./zone/network :prefix "" :export true)
(import ./zone/rctl :prefix "" :export true)
(import ../collector)

(def doer :zone)
(def description "Create and destroy zones. Existing zones cannot be modified.")
(def name-is "Zone name")
(def mandatory-props-ensure
  {:brand {:types [:string]
           :help "Zone brand"}})
(def optional-props-ensure
  {:attr
   {:types [:array]
    :help "See zone/attr"}
   :autoboot
   {:types [:boolean]
    :help "Boot the zone on system boot"}
   :bhyve
   {:types [:table]
    :help "See zone/bhyve"}
   :boot-after-install
   {:types [:string]
    :help "Boot the zone once it is installed"}
   :bootstrap
   {:types [:table]
    :help "See zone/bootstrap"}
   :bootstrap-from
   {:types [:table]
    :help "Copy gurp into the zone, and apply the given file, relative to zone root"}
   :capped-memory
   {:types [:struct]
    :help "Set memory cap. Keys must be :physical and :swap, values are strings like '4G'"}
   :clone-from
   {:types [:string]
    :help "Instead of installing, clone from the given zone, which must exist and be halted"}
   :copy-in
   {:types [:struct]
    :help "Copy files into the zone. Key (keyword) is src, val is dest, relative to zone root. Unqualified src is assumed to be in ../files/"}
   :datasets
   {:types [:tuple]
    :help "ZFS datasets (as strings) to be delegated to zone"}
   :dns
   {:types [:struct]
    :help "DNS info. :domain is a string; :nameservers a tuple of strings"}
   :exec-in
   {:types [:tuple]
    :help "Runs the given commands (:string) in the zone after booting"}
   :final-state
   {:types [:string]
    :help "Put the zone in the given state. Also accepts 'reboot'"}
   :fs
   {:types [:array]
    :help "See zone/fs"}
   :ip-type
   {:types [:string]
    :help "IP type: exclusive or shared"}
   :hostid
   {:types [:string]
    :help "Force this hostid for the zone"}
   :limitpriv
   {:types [:tuple]
    :help "List of privileges to add to zone"}
   :lx-image
   {:types [:string]
    :help "Install zone using this image. See docs for pattern rules"}
   :net
   {:types [:array]
    :help "See zone/network"}
   :pool
   {:types [:string]
    :help "Resource pool to which zone should belong"}
   :rctl
   {:types [:array]
    :help "See zone/rctl"}
   :recreate
   {:types [:number]
    :help "1-in-n chance the zone will be destroyed and recreated"}
   :zonepath
   {:types [:string]
    :help "Path to zone root"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:autoboot true
   :recreate 0
   :boot-after-install true})
(def defaults-remove {})

(defn ensure
  "Given a zone name and spec, put an ensure struct in the collector"
  [name & spec]
  (var modified-spec spec)
  (expand-resource :net)
  (expand-resource :attr)
  (expand-resource :fs)
  (expand-resource :rctl)
  (expand-resource :bhyve :as-struct true)
  (expand-resource :bootstrap :as-struct true)

  (def modified-spec-struct (make-spec-struct ;modified-spec))
  (def spec-struct (checked-spec modified-spec-struct mandatory-props-ensure optional-props-ensure))
  (def spec-table (spec-with-defaults defaults-ensure spec-struct))

  (if-let [copy-resource (get spec-table :copy-in)]
    (set (spec-table :copy-in)
         (zipcoll (map qualify-from-path (keys copy-resource))
                  (values copy-resource))))

  # Fill-in the zone path if it hasn't been given
  (if-not (has-key? spec-table :zonepath)
    (set (spec-table :zonepath) (pathcat "/zones" name)))

  (collector/push :ensure doer (spec->resource doer name spec-table)))

(defn remove
  "Given a zone name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))
