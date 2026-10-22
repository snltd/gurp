(use ./lib)
(use ../dsl)
(import ./zone/attr :prefix "" :export true)
(import ./zone/bhyve :prefix "" :export true)
(import ./zone/bootstrap :prefix "" :export true)
(import ./zone/cloudinit :prefix "" :export true)
(import ./zone/emu :prefix "" :export true)
(import ./zone/fs :prefix "" :export true)
(import ./zone/network :prefix "" :export true)
(import ./zone/rctl :prefix "" :export true)
(import ../collector)
(import ../doer-components :as component)

(def checksum-types ["sha256"])

(defdoer :zone
  "Create and destroy zones. Existing zones cannot be modified."
  :name-is "Zone name"

  :mandatory-props-ensure
  {:brand {:types [:string]
           :help "Zone brand"}
   :autoboot {:types [:boolean]
              :help "Boot the zone when the system boots"
              :default true}
   :boot-after-install {:types [:boolean]
                        :help "Boot the zone once it is installed"
                        :default true}}

  :optional-props-ensure
  {:attr {:types [:array]
          :help "See zone/attr"}
   :bhyve {:types [:table] :help "See zone/bhyve"}
   :bootstrap {:types [:table]
               :help "See zone/bootstrap"}
   :bootstrap-from {:types [:table]
                    :help "Copy gurp into the zone, and apply the given file,
                           relative to zone root"}
   :capped-memory {:types [:struct]
                   :help "Set memory cap"
                   :definition
                   {:physical {:types [:string]
                               :help "Physical memory cap, with suffix. e.g.
                                      '2G'"}
                   :swap {:types [:string]
                          :help "Swap cap, with suffix. e.g. '4G'"}}}
   :clone-from {:types [:string]
                :help "Instead of installing, clone from the given zone, which
                       must exist and be halted"}
   :cloudinit {:types [:array] :help "See zone/cloudinit"}
   :copy-in {:types [:struct]
             :help "Copy files into the zone. Key is source, value is dest,
                    relative to zone root. If key is an array of strings, all
                    files listed are copied to dest. Unqualified src is assumed
                    to be in `../files`. Directories are copied recursively, and
                    if the dest directory does not exist, it is created."}
   :datasets {:types [:tuple]
              :help "ZFS datasets (as strings) to be delegated to zone"}
   :dns {:types [:struct]
         :help "DNS client configuration"
         :definition
         {:domain {:types [:string]
                   :help "Default DNS domain"}
          :nameservers {:types [:array :tuple]
                        :help "Nameserver list"}}}
   :emu {:types [:table] :help "See zone/emu"}
   :exec-in {:types [:tuple]
             :help "Runs the given commands (`:string`) in the zone after
                    booting"}
   :final-state {:types [:string]
                 :help "Put the zone in the given state. Also accepts 'reboot'"}
   :fs {:types [:array]
        :help "See zone/fs"}
   :ip-type {:types [:string]
             :help "IP type"
             :values ["exclusive" "shared"]}
   :hostid {:types [:string]
            :help "Force this hostid for the zone"}
   :limitpriv {:types [:tuple]
               :help "List of privileges to add to zone"}
   :image {:types [:string]
           :help "Install zone using this image. See docs for pattern rules"}
   :image-checksum (component/checksum)
   :network {:types [:array]
             :help "See zone/network"}
   :pool {:types [:string]
          :help "Resource pool to which zone should belong"}
   :rctl {:types [:array]
          :help "See zone/rctl"}
   :recreate {:types [:number]
              :help "1-in-n chance the zone will be destroyed and recreated"}
   :zonepath {:types [:string]
              :help "Path to zone root. Defaults to `/zones/<zone-name>`"}}

  :notes
  ["`kvm` zones are not supported."
   "You cannot modify an existing zone in-place."
   "When building an illumos zone, you can use the `:image` key to specify
    a fully qualified path or URL to the install image. If you do not specify an
    image, Gurp will fetch the latest OmniOS non-global zone ZFS dataset. Images
    are cached in `/var/tmp`."
   "`recreate` must be an integer, and it is the n:1 odds of a zone being
    destroyed and recreated. So, `0` means \"never recreate this zone\", and `1`
    means \"recreate this zone on every run\". You can set the number as high
    as you like, so if you run Gurp every 15 minutes and want your zone rebuilt
    from scratch about once a week, you'd use `:recreate 672`."])

(defn- squash-cloudinit
  "Merges multiple cloudinit resources into a single struct"
  [resources]
  (let [ret @{:from @{} :from-struct @{}}]
    (loop [res :in resources]
      (when-let [val (res :from)]
        (put (ret :from) (res :name) val))
      (when-let [val (res :from-struct)]
        (put (ret :from-struct) (res :name) val)))
    ret))

(defn ensure
  "Given a zone name and spec, put an ensure struct in the collector"
  [name & spec]
  (var modified-spec spec)
  (expand-resource :network)
  (expand-resource :attr)
  (expand-resource :fs)
  (expand-resource :rctl)
  (expand-resource :bhyve :as-struct true)
  (expand-resource :bootstrap :as-struct true)
  (expand-resource :cloudinit)
  (expand-resource :emu :as-struct true)

  (let [modified-spec-struct (make-spec-struct ;modified-spec)
        spec-table (spec-with-defaults (extract-defaults mandatory-props-ensure)
                                       modified-spec-struct)
        spec-struct (pinpoint-error
                      :ensure
                      (checked-spec spec-table
                                    mandatory-props-ensure
                                    optional-props-ensure))]

    (when-let [image-sum-resource (get spec-table :image-checksum)]
      (pinpoint-error
        :ensure
        (checked-spec image-sum-resource
                      {:type {:types [:string]}
                       :value {:types [:string]}}
                      {})))

    (when-let [copy-resource (get spec-table :copy-in)
               expanded-resource (expand-list-struct copy-resource)]
      (set (spec-table :copy-in)
           (zipcoll (map qualify-from-path (keys expanded-resource))
                    (values expanded-resource))))

    (when-let [cloudinit-resources (get spec-table :cloudinit)]
      (set (spec-table :cloudinit) (squash-cloudinit cloudinit-resources)))

    # Fill-in the zone path if it hasn't been given
    (if-not (has-key? spec-table :zonepath)
      (set (spec-table :zonepath) (pathcat "/zones" name)))

    (collector/push :ensure doer (spec->resource doer name spec-table))))

(make-remove-fn "zone")
