fn hex2nibble(hex : u8) -> Result<u8, ()> {
    match hex {
        0x30u8 ... 0x39u8 => Ok(hex - 0x30u8),
        0x41u8 ... 0x46u8 | 0x61u8 ... 0x66u8 => Ok((hex & !0x20u8) - 0x37),
        _ => Err(()),
    }
}

fn hex2octets(hex: Vec<u8>) -> Vec<u8> {
    let mut octets : Vec<u8> = Vec::with_capacity(&hex.len() / 2 + &hex.len() % 2);
    if hex.len() % 2 == 1 {
        octets.push(hex2nibble(hex[0]).unwrap())
    }
    for octet in hex[&hex.len() % 2 ..].chunks(2) {
        octets.push((hex2nibble(octet[0]).unwrap() << 4) + hex2nibble(octet[1]).unwrap())
    }
    octets
}

fn nibble2hex(nibble : u8) -> Result<u8, ()> {
    match nibble {
        0u8 ... 9u8 => Ok(nibble + 0x30u8),
        10u8 ... 15u8 => Ok(nibble + 0x57u8),
        _ => Err(()),
    }
}

fn octets2hex(octets: Vec<u8>) -> Vec<u8> {
    let mut hex : Vec<u8> = Vec::with_capacity(octets.len() * 2);
    for octet in octets {
        hex.push(nibble2hex((octet & 0xF0u8) >> 4).unwrap());
        hex.push(nibble2hex(octet & 0x0Fu8).unwrap());
    }
    hex
}

fn fixed_xor(left: Vec<u8>, right: Vec<u8>) -> Vec<u8> {
    assert!(left.len() == right.len());
    let left_octets = hex2octets(left);
    let right_octets = hex2octets(right);
    let mut xor : Vec<u8> = Vec::with_capacity(left_octets.len());
    for i in 0 .. xor.capacity() {
        xor.push(left_octets[i] ^ right_octets[i])
    }
    xor
}

fn main() {
    let left = String::from("1c0111001f010100061a024b53535009181c").into_bytes();
    let right = String::from("686974207468652062756c6c277320657965").into_bytes();
    println!("{}", String::from_utf8(octets2hex(fixed_xor(left, right))).unwrap());
}