(use judge)
(use ../src/gurp)

(deftest list-doers
  (test
    (list-doers)
    "                    apk  Install and uninstall APK packages. Only valid in an\n                         Alpine LX zone.\n                   cron  Manage cron jobs. Crontab entries are prefixed with a\n                         machine-generated string.\n              directory  Create and remove directories. Parents are created like\n                         mkdir -p, but with the owner/group/mode of the gurp\n                         process. Removal always removes directory contents.\n              etherstub  Create and destroy etherstubs.\n              file-line  Ensure lines do or do not exist in the given file.\n                   file  Create files from multiple sources, or remove them.\n                    gem  Install and uninstall Ruby gems.\n                  group  Create and destroy Unix groups.\n             ip-address  Manages IP addresses via ipadm.\n           ip-interface  Create and destroy IP interfaces, with optional\n                         properties. Properties are supplied with\n                         'ip-interface-protocol'.\n          ip-properties  Sets global IP properties, via 'ipadm set-prop'.\n                  ipnat  Set or remove NAT rules.\n                   misc  A collection of things too small to deserve their own\n                         doer.\n           network-flow  Manage network flows via flowadm.\n                    pkg  Install and uninstall pkg(5) packages.\n                  pkgin  Install and uninstall pkgin packages. Only valid in a\n                         pkgsrc zone.\n              publisher  Add and remove pkg(5) publisher origins.\n                  route  Manage routes. Note that default routes for zones should\n                         be handled by the zone's :defrouter property.\n                    smf  Create and install a manifest for an SMF service.\n                    svc  Manage the state of an existing SMF service.\n                svcprop  Manage properties of an existing SMF service.\n                symlink  Create and remove symbolic links.\n                   user  Manage Unix users\n                   vlan  Manage VLAN objects\n                   vnic  Manage VNIC objects\n                    zfs  Create, destroy, and modify properties of ZFS\n                         filesystems.\n                   zone  Create and destroy zones. Existing zones cannot be\n                         modified."))

(deftest help-for-doer
  (test
    (help-for-doer "directory")
    "\e[1m\e[4mdirectory\e[0m\e[0m\n  Create and remove directories. Parents are created like mkdir -p, but with the\n  owner/group/mode of the gurp process. Removal always removes directory\n  contents.\n\n\e[1m\e[4mdirectory/ensure\e[0m\e[0m\n  \e[1mname\e[0m  [:string]  Fully qualified path to directory\n\n\e[1mMandatory properties\e[0m\n  \e[1mowner\e[0m [:string :number]  The username or UID of the user who owns this\n                           directory\n  \e[1mgroup\e[0m [:string :number]  The group name or GID of the for this\n                           directory\n  \e[1mmode\e[0m  [:string]          Permissions, written as a four-digit octal\n\n\e[1mOptional properties\e[0m\n  None\n\n\e[1m\e[4mdirectory/remove\e[0m\e[0m\n\n\e[1mMandatory properties\e[0m\n  None\n\n\e[1mOptional properties\e[0m\n  None\n"))

(deftest help-for-sub-resource
  (test
    (help-for-sub-resource :zone :network)
    "\e[1m\e[4mzone/network\e[0m\e[0m\n  Describe network configuration of a zone resource.\n\n  \e[1mname\e[0m  [:string]  Zone VNIC, which may already exist\n\n\e[1mMandatory properties\e[0m\n  \e[1mphysical\e[0m [:string]  Zone VNIC. This is the name of the resource, and is\n                      not specified with a key\n\n\e[1mOptional properties\e[0m\n  \e[1mglobal-nic\e[0m      [:string]  Physical NIC on which to create zone VNIC\n  \e[1mallowed-address\e[0m [:string]  IP address, with /netmask\n  \e[1mdefrouter\e[0m       [:string]  IP address of default router\n"))

(deftest doer-lookup
  (test
    (doer-lookup :directory :description)
    "Create and remove directories. Parents are created like mkdir -p, but with   the owner/group/mode of the gurp process. Removal always removes directory   contents.")

  (test
    (doer-lookup :zone :description-fs)
    "Define a filesystem mapping when creating a zone.")

  (test-error
    (doer-lookup :wat :nothing)
    "unknown symbol wat/nothing"))
