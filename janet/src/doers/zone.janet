(use ./definers)
(use ../dsl)
(import ./zone/attr :prefix "" :export true)
(import ./zone/bhyve :prefix "" :export true)
(import ./zone/capped-memory :prefix "" :export true)
(import ./zone/dns :prefix "" :export true)
(import ./zone/bootstrap :prefix "" :export true)
(import ./zone/cloudinit :prefix "" :export true)
(import ./zone/emu :prefix "" :export true)
(import ./zone/fs :prefix "" :export true)
(import ./zone/network :prefix "" :export true)
(import ./zone/rctl :prefix "" :export true)
(import ./file/checksum :prefix "" :export true)
(import ../collector)

(defdoer :zone
  "Create and destroy zones. Existing zones cannot be modified."
  :name-is "Zone name"
  (ensure
    (property "brand" [:string] :mandatory "Zone brand")
    (property "autoboot" [:boolean] :mandatory
              "Boot the zone when the system boots"
              :default true)
    (property "boot-after-install" [:boolean] :mandatory
              "Boot the zone once it is installed"
              :default true)
    (property "attr" [:array] :optional "See zone/attr")
    (property "bhyve" [:struct] :optional "See zone/bhyve")
    (property "bootstrap" [:struct] :optional "See zone/bootstrap")
    (property "capped-memory" [:struct] :optional "See zone/capped-memory")
    (property "clone-from" [:string] :optional
              "Instead of installing, clone from the given zone, which must
               exist and be halted")
    (property "cloudinit" [:array] :optional "See zone/cloudinit")
    (property "copy-in" [:struct] :optional
              "Copy files into the zone. Key is source, value is dest, relative
               to zone root. If key is an array of strings, all files listed are
               copied to dest. Unqualified src is assumed to be in `../files`.
               Directories are copied recursively, and if the dest directory
               does not exist, it is created.")
    (property "datasets" [:tuple] :optional
              "ZFS datasets (as strings) to be delegated to zone")
    (property "dns" [:struct] :optional "See zone/dns")
    (property "emu" [:struct] :optional "See zone/emu")
    (property "exec-in" [:tuple] :optional
              "Runs the given commands (`:string`) in the zone after booting")
    (property "final-state" [:string] :optional
              "Put the zone in the given state. Also accepts 'reboot'")
    (property "fs" [:array] :optional "See zone/fs")
    (property "ip-type" [:string] :optional "IP type"
              :values ["exclusive" "shared"])
    (property "hostid" [:string] :optional "Force this hostid for the zone")
    (property "limitpriv" [:tuple] :optional "List of privileges to add to zone")
    (property "image" [:string] :optional
              "Install zone using this image. See docs for pattern rules")
    (property "checksum" [:struct] :optional "Image checksum. See file/checksum")
    (property "network" [:array] :optional "See zone/network")
    (property "pool" [:string] :optional
              "Resource pool to which zone should belong")
    (property "rctl" [:tuple] :optional "See zone/rctl")
    (property "recreate" [:number] :optional
              "1-in-n chance the zone will be destroyed and recreated")
    (property "zonepath" [:string] :optional
              "Path to zone root. Defaults to `/zones/<zone-name>`"))
  (notes
    "`kvm` zones are not supported."
    "You cannot modify an existing zone in-place."
    "When building an illumos zone, you can use the `:image` key to specify
     a fully qualified path or URL to the install image. If you do not specify an
     image, Gurp will fetch the latest OmniOS non-global zone ZFS dataset. Images
     are cached in `/var/tmp`."
    "`recreate` must be an integer, and it is the n:1 odds of a zone being
     destroyed and recreated. So, `0` means \"never recreate this zone\", and `1`
     means \"recreate this zone on every run\". You can set the number as high
     as you like, so if you run Gurp every 15 minutes and want your zone rebuilt
     from scratch about once a week, you'd use `:recreate 672`."))

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
  (pinpoint-error
    :ensure
    (var modified-spec spec)
    (expand-resource :network)
    (expand-resource :attr)
    (expand-resource :fs)
    (expand-resource :rctl)
    (expand-resource :bhyve :as-struct true)
    (expand-resource :bootstrap :as-struct true)
    (expand-resource :cloudinit)
    (expand-resource :emu :as-struct true)
    (expand-resource :dns :as-struct true)
    (expand-resource :capped-memory :as-struct true)

    (let [modified-spec-struct (make-spec-struct ;modified-spec)
          spec-table (spec-with-defaults
                       (extract-defaults mandatory-props-ensure)
                       modified-spec-struct)
          spec-struct (checked-spec spec-table
                                    mandatory-props-ensure
                                    optional-props-ensure)]

      (when-let [image-sum-resource (get spec-table :image-checksum)]
        (checked-spec image-sum-resource
                      {:type {:types [:string]}
                       :value {:types [:string]}}
                      {}))

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

      (collector/push :ensure doer (spec->resource doer name spec-table)))))

(make-fn :remove :zone)
