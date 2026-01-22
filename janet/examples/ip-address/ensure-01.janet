(ip-address/ensure "test0/v4"
                   :type "static"
                   :address "192.168.1.13/24"
                   :properties {:prefixlen 24
                                :transmit true
                                :private false})
