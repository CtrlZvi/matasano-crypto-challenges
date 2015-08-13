fn hex2nibble(hex : u8) -> Result<u8, ()> {
    match hex {
        0x30u8 ... 0x39u8 => Ok(hex - 0x30u8),
        0x41u8 ... 0x46u8 | 0x61u8 ... 0x66u8 => Ok((hex & !0x20u8) - 0x37),
        _ => Err(()),
    }
}

fn hex2octets(hex: &Vec<u8>) -> Vec<u8> {
    let mut octets : Vec<u8> = Vec::with_capacity(&hex.len() / 2 + &hex.len() % 2);
    if hex.len() % 2 == 1 {
        octets.push(hex2nibble(hex[0]).unwrap())
    }
    for octet in hex[&hex.len() % 2 ..].chunks(2) {
        octets.push((hex2nibble(octet[0]).unwrap() << 4) + hex2nibble(octet[1]).unwrap());
    }
    octets
}

fn fixed_xor(input: &[u8], key: u8) -> Vec<u8> {
    let mut xor : Vec<u8> = Vec::with_capacity(input.len());
    for octet in input {
        xor.push(octet ^ key)
    }
    xor
}

fn main() {
    let letter_frequencies = vec![0.08167f32, 0.01492f32, 0.02782f32, 0.04253f32, 0.12702f32, 0.02228f32, 0.02015f32, 0.06094f32, 0.06966f32, 0.00153f32, 0.00772f32, 0.04025f32, 0.02406f32, 0.06749f32, 0.07507f32, 0.01929f32, 0.00095f32, 0.05987f32, 0.06327f32, 0.09056f32, 0.02758f32, 0.00978f32, 0.02361f32, 0.00150f32, 0.01974f32, 0.00074f32];
    assert!(letter_frequencies.len() == 26);

    let mut xor_character = 0u8;
    let mut xor_delta = std::f32::INFINITY;

    let input = String::from("1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736").into_bytes();
    for c in 0u8 .. 127u8 {
        let output = fixed_xor(&hex2octets(&input), c);

        let mut letter_count : Vec<f32> = Vec::with_capacity(letter_frequencies.len());
        for _ in 0 .. letter_frequencies.len() {
            letter_count.push(0f32);
        }
        let mut nonletter_count = 0f32;
        for octet in &output {
            match *octet {
                0x41u8 ... 0x5Au8 | 0x61u8 ... 0x7Au8 => { letter_count[((octet | 0x20u8) - 0x61u8) as usize] += 1f32; },
                0x20 => (),
                _ => { nonletter_count += 1f32; },
            }
        }

        let mut cumulative_delta = 0f32;
        for i in 0 .. letter_frequencies.len() {
            cumulative_delta += (letter_frequencies[i] - (letter_count[i] / output.len() as f32)).abs();
        }
        cumulative_delta /= (output.len() as f32 - nonletter_count) / output.len() as f32;
        if cumulative_delta < xor_delta {
            xor_character = c;
            xor_delta = cumulative_delta;
        }
    }

    let output = fixed_xor(&hex2octets(&input), xor_character);
    println!("Character: {}", xor_character);
    println!("Delta: {}", xor_delta);
    println!("String: {}", String::from_utf8(output).unwrap());
}

#[cfg(test)]
mod tests {

    fn nibble2hex(nibble : u8) -> Result<u8, ()> {
       match nibble {
            0u8 ... 9u8 => Ok(nibble + 0x30u8),
            10u8 ... 15u8 => Ok(nibble + 0x57u8),
            _ => Err(()),
        }
    }

    fn octets2hex(octets: &[u8]) -> Vec<u8> {
       let mut hex : Vec<u8> = Vec::with_capacity(octets.len() * 2);
        for octet in octets {
            hex.push(nibble2hex((octet & 0xF0u8) >> 4).unwrap());
            hex.push(nibble2hex(octet & 0x0Fu8).unwrap());
        }
        hex
    }

    #[test]
    fn test_octets2hex() {
        let hex = octets2hex(&vec![10u8]);
        assert!(hex.len() == 2);
        println!("{}, {}", hex[0], hex[1]);
        assert!(octets2hex(&vec![10u8]) == vec!['0' as u8, 'a' as u8]);
    }
}