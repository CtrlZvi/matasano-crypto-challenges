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

fn octets2base64(octets: Vec<u8>) -> Vec<u8> {
    let mut base64 : Vec<u8> = Vec::with_capacity(4 * (octets.len() / 3 + match octets.len() {
        0 => 0,
        _ => 1
    }));
    for chunk in octets.chunks(3) {
        let groups = match chunk.len() {
            1 => [(chunk[0] & 0xFCu8) >> 2, (chunk[0] & 0x03u8) << 4, 0xFFu8, 0xFFu8],
            2 => [(chunk[0] & 0xFCu8) >> 2, ((chunk[0] & 0x03u8) << 4) + ((chunk[1] & 0xF0u8) >> 4), (chunk[1] & 0x0Fu8) << 2, 0xFFu8],
            3 => [(chunk[0] & 0xFCu8) >> 2, ((chunk[0] & 0x03u8) << 4) + ((chunk[1] & 0xF0u8) >> 4), ((chunk[1] & 0x0Fu8) << 2) + ((chunk[2] & 0xC0u8) >> 6), chunk[2] & 0x3Fu8],
            _ => panic!(),
        };
        for group in &groups {
            base64.push(match *group {
                0x00u8 ... 0x19u8 => group + 0x41u8,
                0x1Au8 ... 0x33u8 => group + 0x47u8,
                0x34u8 ... 0x3Du8 => group - 0x04u8,
                0x3Eu8 => 0x2Bu8,
                0x3Fu8 => 0x2Fu8,
                0xFFu8 => 0x3Du8,
                _ => panic!(),
            })
        }
    }
    base64
}

fn hex2base64(hex: Vec<u8>) -> Vec<u8> {
    octets2base64(hex2octets(hex))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hex2nibble() {
        assert!(::hex2nibble('1' as u8) == Ok(1u8));
        assert!(::hex2nibble('2' as u8) == Ok(2u8));
        assert!(::hex2nibble('3' as u8) == Ok(3u8));
        assert!(::hex2nibble('4' as u8) == Ok(4u8));
        assert!(::hex2nibble('5' as u8) == Ok(5u8));
        assert!(::hex2nibble('6' as u8) == Ok(6u8));
        assert!(::hex2nibble('7' as u8) == Ok(7u8));
        assert!(::hex2nibble('8' as u8) == Ok(8u8));
        assert!(::hex2nibble('9' as u8) == Ok(9u8));
        assert!(::hex2nibble('a' as u8) == Ok(10u8));
        assert!(::hex2nibble('A' as u8) == Ok(10u8));
        assert!(::hex2nibble('b' as u8) == Ok(11u8));
        assert!(::hex2nibble('B' as u8) == Ok(11u8));
        assert!(::hex2nibble('c' as u8) == Ok(12u8));
        assert!(::hex2nibble('C' as u8) == Ok(12u8));
        assert!(::hex2nibble('d' as u8) == Ok(13u8));
        assert!(::hex2nibble('D' as u8) == Ok(13u8));
        assert!(::hex2nibble('e' as u8) == Ok(14u8));
        assert!(::hex2nibble('E' as u8) == Ok(14u8));
        assert!(::hex2nibble('f' as u8) == Ok(15u8));
        assert!(::hex2nibble('F' as u8) == Ok(15u8));
        assert!(::hex2nibble('z' as u8) == Err(()));
    }

    #[test]
    fn test_hex2octets() {
        assert!(::hex2octets(String::from("0").into_bytes()) == vec![0u8]);
        assert!(::hex2octets(String::from("a").into_bytes()) == vec![10u8]);
        assert!(::hex2octets(String::from("10").into_bytes()) == vec![16u8]);
        assert!(::hex2octets(String::from("1a").into_bytes()) == vec![26u8]);
        assert!(::hex2octets(String::from("100").into_bytes()) == vec![1u8, 0u8]);
        assert!(::hex2octets(String::from("1a00").into_bytes()) == vec![26u8, 0u8]);
        assert!(::hex2octets(String::from("12345").into_bytes()) == vec![1u8, 35u8, 69u8]);

    }

    #[test]
    fn test_octets2base64() {
        assert!(::octets2base64(vec![0u8]) == vec!['A' as u8, 'A' as u8, '=' as u8, '=' as u8]);
        assert!(::octets2base64(vec![128u8]).len() == 4);
        assert!(::octets2base64(vec![128u8]) == vec!['g' as u8, 'A' as u8, '=' as u8, '=' as u8]);
        assert!(::octets2base64(vec![0u8, 0u8]) == vec!['A' as u8, 'A' as u8, 'A' as u8, '=' as u8]);
        assert!(::octets2base64(vec![1u8, 35u8, 69u8]) == vec!['A' as u8, 'A' as u8, 'A' as u8, 'A' as u8]);

    }
}

fn main() -> () {
    let base64 = hex2base64(String::from("49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d").into_bytes());
    println!("{}", String::from_utf8(base64).unwrap());
}