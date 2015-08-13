use std::fs::File;
use std::io::Read;

fn base64decode(input : &[u8]) -> Vec<u8> {
    let mut octets : Vec<u8> = Vec::new();

    let mut input_octets = input.iter().map(|c| match *c {
        0x41u8 ... 0x5Au8 => c - 0x41u8,
        0x61u8 ... 0x7Au8 => c - 0x47u8,
        0x30u8 ... 0x39u8 => c + 0x04u8,
        0x2Bu8 => 0x3Eu8,
        0x2Fu8 => 0x3Fu8,
        0x3Du8 => 0xFFu8,
        i => panic!("{}", i),
    }).collect::<Vec<u8>>();
    input_octets.retain(|c| *c != 0xFFu8);
    for chunk in input_octets.chunks(4) {
        let groups = match chunk.len() {
            2 => vec![((chunk[0] << 2) & 0xFCu8) + ((chunk[1] >> 4) & 0x03u8)],
            3 => vec![((chunk[0] << 2) & 0xFCu8) + ((chunk[1] >> 4) & 0x03u8), ((chunk[1] << 4) & 0xF0u8) + ((chunk[2] >> 2) & 0x0F)],
            4 => vec![((chunk[0] << 2) & 0xFCu8) + ((chunk[1] >> 4) & 0x03u8), ((chunk[1] << 4) & 0xF0u8) + ((chunk[2] >> 2) & 0x0F), ((chunk[2] << 6) & 0xC0u8) + (chunk[3] & 0x3Fu8)],
            _ => panic!(),
        };
        octets.reserve(groups.len());
        for group in groups {
            octets.push(group)
        }
    }

    octets
}

fn hamming_distance(left : &[u8], right : &[u8]) -> u32 {
    assert!(left.len() == right.len());
    let mut distance = 0u32;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        let mut xor = left_byte ^ right_byte;
        for _ in 0 .. 8 {
            distance += xor as u32 & 1;
            xor >>= 1;
        }
    }
    distance
}

fn find_keysize(input : &[u8]) -> Vec<(usize, f32)> {
    let mut keys : Vec<(usize, f32)> = Vec::with_capacity(input.len() / 4);
    for keysize in (1 .. input.len() / 4 + 1) {
        let mut iter = input.chunks(keysize);

        let normalized_distance = vec![hamming_distance(iter.next().unwrap(), iter.next().unwrap()) as f32 / keysize as f32, hamming_distance(iter.next().unwrap(), iter.next().unwrap()) as f32 / keysize as f32];
        let mut average_distance = 0f32;
        for distance in normalized_distance.iter() {
            average_distance += *distance;
        }
        average_distance /= normalized_distance.len() as f32;
        keys.push((keysize, average_distance));
    }
    keys.sort_by(|&(_, distance_a), &(_, distance_b)| distance_a.partial_cmp(&distance_b).unwrap());
    keys
}

fn fixed_xor(input: &[u8], key: u8) -> Vec<u8> {
    let mut xor : Vec<u8> = Vec::with_capacity(input.len());
    for octet in input {
        xor.push(octet ^ key)
    }
    xor
}

fn histogram(input : &[u8], key : u8) -> f32 {
    let letter_frequencies = vec![0.08167f32, 0.01492f32, 0.02782f32, 0.04253f32, 0.12702f32, 0.02228f32, 0.02015f32, 0.06094f32, 0.06966f32, 0.00153f32, 0.00772f32, 0.04025f32, 0.02406f32, 0.06749f32, 0.07507f32, 0.01929f32, 0.00095f32, 0.05987f32, 0.06327f32, 0.09056f32, 0.02758f32, 0.00978f32, 0.02361f32, 0.00150f32, 0.01974f32, 0.00074f32];
    assert!(letter_frequencies.len() == 26);

    let output = fixed_xor(input, key);

    let mut letter_count : Vec<f32> = Vec::with_capacity(letter_frequencies.len());
    for _ in 0 .. letter_frequencies.len() {
        letter_count.push(0f32);
    }
    let mut nonletter_count = 0f32;
    let mut cumulative_delta = 0f32;
    for octet in &output {
        match *octet {
            0x41u8 ... 0x5Au8 | 0x61u8 ... 0x7Au8 => { letter_count[((octet | 0x20u8) - 0x61u8) as usize] += 1f32; },
            0x20 => (),
            0x80u8 ... 0xFFu8 => { cumulative_delta = std::f32::INFINITY; }
            _ => { nonletter_count += 1f32; },
        }
    }

    for i in 0 .. letter_frequencies.len() {
        cumulative_delta += (letter_frequencies[i] - (letter_count[i] / output.len() as f32)).abs();
    }
    let ratio = (output.len() as f32 - nonletter_count) / output.len() as f32;
    cumulative_delta / (ratio * ratio)
}

fn decrypt_fixed_xor(input : &[u8]) -> (u8, f32) {

    let mut best_key = 0u8;
    let mut best_score = std::f32::INFINITY;

    for key in 0u8 .. std::u8::MAX {
        let score = histogram(input, key);
        if score < best_score {
            best_key = key;
            best_score = score;
        }
    }

    (best_key, best_score)
}

fn find_repeating_xor_key(input : &[u8], keysize : usize) -> Vec<(u8, f32)> {
    let mut key : Vec<(u8, f32)> = Vec::with_capacity(keysize);
    let mut data : Vec<Vec<u8>> = Vec::with_capacity(keysize);
    for i in 0 .. data.capacity() {
        data.push(Vec::with_capacity(input.len() / keysize + match i <= input.len() % keysize {
            true => 1,
            false => 0,
        }));
    }
    for chunk in input.chunks(keysize) {
        for (datum, vec) in chunk.iter().zip(data.iter_mut()) {
            vec.push(*datum);
        }
    }
    for octets in data {
        let (c, delta) = decrypt_fixed_xor(&octets);
        key.push((c, delta));
    }
    key
}

fn rotating_xor(input: &[u8], key: &[u8]) -> Vec<u8> {
    let mut xor : Vec<u8> = Vec::with_capacity(input.len());
    for (octet, byte) in input.iter().zip(key.iter().cycle()) {
        xor.push(octet ^ byte)
    }
    xor
}

fn main() {
    let mut f = File::open("6.txt").unwrap();
    let mut text = Vec::new();
    match f.read_to_end(&mut text) {
        Ok(_) => (),
        Err(_) => panic!(),
    };
    text.retain(|&c| c as char != '\n');
    let data = base64decode(&text);
    let keysizes = find_keysize(&data);
    let key_attempts = 10;
    let mut key_options : Vec<(Vec<u8>, f32)> = Vec::with_capacity(key_attempts);
    for keysize in keysizes.iter().take(key_attempts) {
        let key = find_repeating_xor_key(&data, keysize.0);
        let (keys, deltas) : (Vec<u8>, Vec<f32>) = key.into_iter().unzip();
        let mut sum = 0f32;
        for x in &deltas {
            sum += *x;
        }
        key_options.push((keys, sum / deltas.len() as f32));
    }
    key_options.sort_by(|&(_, delta_a), &(_, delta_b)| delta_a.partial_cmp(&delta_b).unwrap());

    let key = &key_options.first().unwrap().0;
    let score = &key_options.first().unwrap().1;
    println!("Key size: {}", key.len());
    println!("Score: {}", score);
    println!("{}", String::from_utf8(rotating_xor(&data, key)).unwrap());
}

#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::io::Read;

    fn base64encode(octets: &[u8]) -> Vec<u8> {
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

    #[test]
    fn test_base64decode() {
        let input = String::from("SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t");
        assert!(String::from_utf8(base64encode(&::base64decode(input.as_bytes()))).unwrap() == input);

        let mut f = File::open("6.txt").unwrap();
        let mut text = Vec::new();
        match f.read_to_end(&mut text) {
            Ok(_) => (),
            Err(_) => panic!(),
        };
        text.retain(|&c| c as char != '\n');
        let recoded = String::from_utf8(base64encode(&::base64decode(&text))).unwrap();
        let original = String::from_utf8(text).unwrap();
        println!("Original: {}", original);
        println!("Recoded: {}", recoded);
        assert!(original == recoded);
    }
}