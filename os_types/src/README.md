   For subcommands that take an addrobj, the addrobj specifies a unique
     address on the system, and must be unique itself.  It is made up of two
     parts, delimited by a ‘/’.  The first part is the name of the interface
     and the second part is an arbitrary string up to 32 alphanumeric
     characters long, where the first character must be alphabetic (e.g.
     a-z,A-Z).  For example, "lo0/v4" is a loopback interface addrobj name,
     which could also be called "lo0/ipv4loopback".  Consumers should note
     that this length limit may be lifted in the future.

     For subcommands that take a protocol, this can be one of the following
     values: ip, ipv4, ipv6, icmp, tcp, sctp or udp.
