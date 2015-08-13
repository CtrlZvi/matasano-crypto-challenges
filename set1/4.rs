use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

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

fn decrypt_xor(input : &Vec<u8>) -> (u8, f32) {
    let letter_frequencies = vec![0.08167f32, 0.01492f32, 0.02782f32, 0.04253f32, 0.12702f32, 0.02228f32, 0.02015f32, 0.06094f32, 0.06966f32, 0.00153f32, 0.00772f32, 0.04025f32, 0.02406f32, 0.06749f32, 0.07507f32, 0.01929f32, 0.00095f32, 0.05987f32, 0.06327f32, 0.09056f32, 0.02758f32, 0.00978f32, 0.02361f32, 0.00150f32, 0.01974f32, 0.00074f32];
    assert!(letter_frequencies.len() == 26);

    let mut xor_character = 0u8;
    let mut xor_delta = std::f32::INFINITY;

    for c in 0u8 .. 127u8 {
        let output = fixed_xor(&input, c);

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

    (xor_character, xor_delta)
}

fn main() {
    let f = File::open("4.txt").unwrap();
    let b = BufReader::new(&f);
    let mut best_line = String::from("");
    let mut key = 0u8;
    let mut best_delta = std::f32::INFINITY;
    for line in b.lines() {
        let octets = line.unwrap().into_bytes();
        let (c, delta) = decrypt_xor(&hex2octets(&octets));
        if delta < best_delta {
            best_delta = delta;
            key = c;
            best_line = String::from_utf8(octets).unwrap();
        }
    }

    println!("Line: {}", best_line);
    println!("Key: {}", key);
    println!("Delta: {}", best_delta);

    let input = hex2octets(&best_line.into_bytes());
    println!("Decrypted: {}", String::from_utf8(fixed_xor(&input, key)).unwrap());
}

#[cfg(test)]
mod tests {
}