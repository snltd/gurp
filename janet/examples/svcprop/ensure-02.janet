(svcprop/ensure "mariadb"
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"})
