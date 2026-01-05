
// Should be same as your pc is connected to
const WIFI_NETWORK: &str = "";
const WIFI_PASSWORD: &str = "";

// IP address octets and prefix length (single source of truth)
// Needs to be unique and in the same subnet as your pc
const ADDRESS_OCTETS: [u8; 4] = [0, 0, 0, 0];

// change to the correct CIDR Prefix:
const ADDRESS_PREFIX: u8 = 24; 


// Gateway octets
// Needs to be unique and in the same subnet as your pc
const GATEWAY_OCTETS: [u8; 4] = [0, 0, 0, 0];

// Only change the port when the network environment 
// requires a different TCP port — e.g. the host has a conflict, 
// or routing/firewall/NAT forces a different external port.
// Pico port for host TCP connection
const PICO_PORT: u16 = 1234;

