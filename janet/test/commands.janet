(use judge)
(use ../src/gurp)

(deftest list-doers
  (test
    (list-doers false)
    "                       \e[1mapk\e[0m  Install and uninstall APK packages. Only\n                            valid in an Alpine LX zone.\n                    \e[1mbridge\e[0m  Create and modify ethernet bridges.\n                      \e[1mcron\e[0m  Manage cron jobs. Crontab entries are\n                            prefixed with a machine-generated string.\n                 \e[1mdirectory\e[0m  Create and remove directories. Parents are\n                            created like mkdir -p, but with the\n                            owner/group/mode of the gurp process. Removal\n                            always removes directory contents.\n                 \e[1metherstub\e[0m  Create and destroy etherstubs.\n                 \e[1mfile-line\e[0m  Ensure lines do or do not exist in the\n                            given file.\n                      \e[1mfile\e[0m  Create files from multiple sources, or\n                            remove them.\n                       \e[1mgem\e[0m  Install and uninstall Ruby gems.\n                     \e[1mgroup\e[0m  Create and destroy Unix groups.\n                \e[1mip-address\e[0m  Manages IP addresses via ipadm.\n              \e[1mip-interface\e[0m  Create and destroy IP interfaces, with\n                            optional properties. Properties are supplied with\n                            'ip-interface-protocol'.\n             \e[1mip-properties\e[0m  Sets global IP properties, via 'ipadm\n                            set-prop'.\n                     \e[1mipnat\e[0m  Set or remove NAT rules.\n                      \e[1mmisc\e[0m  A collection of things too small to deserve\n                            their own doer.\n              \e[1mnetwork-flow\e[0m  Manage network flows via flowadm.\n                       \e[1mpkg\e[0m  Install and uninstall pkg(5) packages.\n                     \e[1mpkgin\e[0m  Install and uninstall pkgin packages. Only\n                            valid in a pkgsrc zone.\n                 \e[1mpublisher\e[0m  Add and remove pkg(5) publisher origins.\n                     \e[1mroute\e[0m  Manage routes. Note that default routes for\n                            zones should be handled by the zone's :defrouter\n                            property.\n                       \e[1msmf\e[0m  Create and install a manifest for an SMF\n                            service.\n                       \e[1msvc\e[0m  Manage the state of an existing SMF\n                            service.\n                   \e[1msvcprop\e[0m  Manage properties of an existing SMF\n                            service.\n                   \e[1msymlink\e[0m  Create and remove symbolic links.\n                      \e[1muser\e[0m  Manage Unix users\n                      \e[1mvlan\e[0m  Manage VLAN objects\n                      \e[1mvnic\e[0m  Manage VNIC objects\n                       \e[1mzfs\e[0m  Create, destroy, and modify properties of\n                            ZFS filesystems.\n                      \e[1mzone\e[0m  Create and destroy zones. Existing zones\n                            cannot be modified."))

(deftest help-for-doer
  (test-stdout
    (help-for "directory") `
    [1m[4mdirectory[0m[0m
      Create and remove directories. Parents are created like mkdir -p, but with
      the owner/group/mode of the gurp process. Removal always removes directory
      contents.
    
    [1m[4mdirectory/ensure[0m[0m
      [1mname[0m  [:string]  Fully qualified path to directory
    
    [1mMandatory properties[0m
      [1mowner[0m [:string :number]  The username or UID of the user who owns
                               this directory
      [1mgroup[0m [:string :number]  The group name or GID of the for this
                               directory
      [1mmode[0m  [:string]          Permissions, written as a four-digit octal
    
    [1mOptional properties[0m
      None
    
    [1m[4mdirectory/remove[0m[0m
    
    [1mMandatory properties[0m
      None
    
    [1mOptional properties[0m
      None
    
  `))

(deftest help-for-sub-resource
  (test-stdout
    (help-for "zone/network") `
    [1m[4mzone/network[0m[0m
      Describe network configuration of a zone resource.
    
      [1mname[0m  [:string]  Zone VNIC, which may already exist
    
    [1mMandatory properties[0m
      [1mphysical[0m [:string]  Zone VNIC. This is the name of the resource, and
                          is not specified with a key
    
    [1mOptional properties[0m
      [1mglobal-nic[0m      [:string]  Physical NIC on which to create zone VNIC
      [1mallowed-address[0m [:string]  IP address, with /netmask
      [1mdefrouter[0m       [:string]  IP address of default router
    
  `))
