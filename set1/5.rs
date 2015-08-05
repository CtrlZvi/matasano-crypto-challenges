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

fn rotating_xor(input: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let mut xor : Vec<u8> = Vec::with_capacity(input.len());
    for (octet, byte) in input.iter().zip(key.iter().cycle()) {
        xor.push(octet ^ byte)
    }
    xor
}

fn main() {
    let lines = vec![String::from("Burning 'em, if you ain't quick and nimble\nI go crazy when I hear a cymbal")];
    for line in lines {
        let encrypted = rotating_xor(&line.into_bytes(), &String::from("ICE").into_bytes());
        println!("{}", String::from_utf8(octets2hex(encrypted)).unwrap());
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_octets2hex() {
        let hex = ::octets2hex(vec![10u8]);
        assert!(hex.len() == 2);
        println!("{}, {}", hex[0], hex[1]);
        assert!(::octets2hex(vec![10u8]) == vec!['0' as u8, 'a' as u8]);
    }
}