/// Helper code for Signbus

pub const I2C_MAX_LEN: usize = 255;
pub const HEADER_SIZE: usize = 12;
pub const I2C_MAX_DATA_LEN: usize = I2C_MAX_LEN - HEADER_SIZE;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
	CommandComplete,	
	AddressNak,
	DataNak,
	ArbitrationLost,
}

#[derive(Clone,Copy,PartialEq)]
pub enum MasterAction {
    Read(u8),
    Write,
}

/// Signbus Packet 
#[repr(C, packed)]
pub struct SignbusNetworkFlags {
    pub is_fragment:	bool,
    pub is_encrypted:	bool,
    pub rsv_wire_bit5:	bool,
    pub rsv_wire_bit4:	bool,
    pub version:		u8,
}

#[repr(C, packed)]
pub struct SignbusNetworkHeader {
    pub flags:				SignbusNetworkFlags,
    pub src:				u8,
    pub sequence_number:	u16,
    pub length:				u16,
    pub fragment_offset:	u16,
}

#[repr(C, packed)]
pub struct Packet {
	pub header: SignbusNetworkHeader,
	pub data: [u8; I2C_MAX_DATA_LEN],
}

// packet -> [u8]
pub fn serialize_packet(packet: Packet, data_len: usize, buf: &mut [u8]) {
	// Network Flags	
	buf[0] = packet.header.flags.is_fragment as u8; 
	buf[1] = packet.header.flags.is_encrypted as u8; 
	buf[2] = packet.header.flags.rsv_wire_bit5 as u8; 
	buf[3] = packet.header.flags.rsv_wire_bit4 as u8; 
	buf[4] = packet.header.flags.version;

	// Network Header
	buf[5] = packet.header.src;	
	buf[6] = (packet.header.sequence_number & 0x00FF) as u8;
	buf[7] = ((packet.header.sequence_number & 0xFF00) >> 8) as u8;
	buf[8] = (packet.header.length & 0x00FF) as u8;
	buf[9] = ((packet.header.length & 0xFF00) >> 8) as u8;
	buf[10] = (packet.header.fragment_offset & 0x00FF) as u8;
	buf[11] = ((packet.header.fragment_offset & 0xFF00) >> 8) as u8;

	// Copy packet.data to buf
	for (i, c) in packet.data[0..data_len].iter().enumerate() {
	    buf[i+HEADER_SIZE] = *c;
	}
	
	//debug!("{:?}", buf);
}

// [u8] -> packet
pub fn unserialize_packet(buf: &[u8]) -> Packet {
	// Network Flags
	let flags: SignbusNetworkFlags = SignbusNetworkFlags {
        is_fragment:   	buf[0] == 1, // cannot cast u8 to bool? 
        is_encrypted:   buf[1] == 1,
        rsv_wire_bit5:  buf[2] == 1,
        rsv_wire_bit4:  buf[3] == 1,
        version:        buf[4],
    };

    // Network Header
    let header: SignbusNetworkHeader = SignbusNetworkHeader {
        flags:              flags,
        src:                buf[5],
        sequence_number:    (buf[6] as u16) | (buf[7] as u16) << 8,
        length:             (buf[8] as u16) | (buf[9] as u16) << 8,
        fragment_offset:    (buf[10] as u16) | (buf[11] as u16) << 8,
    };

	//debug!("header.length: {}", header.length);
	//debug!("header.offset: {}", header.fragment_offset);
	
	if header.flags.is_fragment {
		// Copy data from slice to fixed sized array to package into packet
		let mut data: [u8; I2C_MAX_DATA_LEN] = [0; I2C_MAX_DATA_LEN]; 
        for (i, c) in buf[HEADER_SIZE..I2C_MAX_LEN].iter().enumerate() {
        	data[i] = *c;
        }

		// Packet
		Packet {
        	header: header,
        	data:   data,
    	}
	}
	else {
		// Copy data from slice to fixed size array to package into packet
		let end = (header.length - HEADER_SIZE as u16 - header.fragment_offset) as usize;
		let mut data: [u8; I2C_MAX_DATA_LEN] = [0; I2C_MAX_DATA_LEN]; 
        for (i, c) in buf[HEADER_SIZE..end].iter().enumerate() {
        	data[i] = *c;
        }

   		// Packet
		Packet {
        	header: header,
        	data:   data,
    	}
	}
}


