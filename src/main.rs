fn main() {
    // Input symbols
    const ALPHABET_SIZE: u32 = 2; // Only bits for now
    let input = vec![0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // Unpacked, raw symbols

    println!("Alphabet size: {}", ALPHABET_SIZE);
    println!("Input string: {:?} ({} bits)", input, input.len());

    // Symbol frequencies
    let f0 = input.iter().fold(0, |acc, x| acc + (1 - *x));
    let f1 = input.iter().fold(0, |acc, x| acc + *x);

    println!("f0: {}", f0);
    println!("f1: {}", f1);

    // Symbol probabilities (approx.)
    let p0 = (f0 as f64) / (input.len() as f64);
    let p1 = (f1 as f64) / (input.len() as f64);

    println!("p0: ~{}", p0);
    println!("p1: ~{}", p1);

    // Scaled probability sum (determines # of states, coding precision).
    const L: u32 = 16;

    println!("L: {}", L);

    // Scaled integer probabilities (must sum to L)
    let f0_scaled = ((L as f64) * p0).round() as u32;
    let f1_scaled = L - f0_scaled;

    println!("f0 scaled: {}", f0_scaled);
    println!("f1 scaled: {}", f1_scaled);

    // Cumulative freq's (trivial in this case, but written out for practice)
    let b0 = 0;
    let b1 = f0_scaled;

    println!("b0: {}", b0);
    println!("b1: {}", b1);

    // Renormalization range
    println!("I = [{}, {}]", L, L * 2 - 1);

    // Precursor ranges
    println!("I0 = [{}, {}]", f0_scaled, f0_scaled * 2 - 1);
    println!("I1 = [{}, {}]", f1_scaled, f1_scaled * 2 - 1);

    // State count
    const STATE_COUNT: u32 = L * 2;

    println!("State count: {}", STATE_COUNT);

    // Encoding/decoding table/string (the simplest/most naive construction, not necessarily worst)
    let mut encoding_table = vec![0; (STATE_COUNT * ALPHABET_SIZE) as usize];
    let mut decoding_table = vec![(0, 0); STATE_COUNT as usize];
    let mut encoding_string = Vec::new();

    let mut max_state_0 = f0_scaled - 1; // - 1 because we want this to be an inclusive upper bound, and we'll increment it before use during table construction
    let mut max_state_1 = f1_scaled - 1;

    for to_state in L..L * 2 {
        let symbol = if to_state - L < b1 { 0 } else { 1 };
        let from_state = match symbol {
            0 => {
                max_state_0 += 1;
                max_state_0
            }
            _ => {
                max_state_1 += 1;
                max_state_1
            }
        };

        encoding_table[(from_state * ALPHABET_SIZE + symbol) as usize] = to_state;

        decoding_table[to_state as usize] = (symbol, from_state);

        encoding_string.push(symbol);
    }

    println!("Encoding table: {:?}", encoding_table);
    println!("Decoding table: {:?}", decoding_table);
    println!("Encoding string: {}", encoding_string.into_iter().fold(String::new(), |acc, x| format!("{}{}", acc, x)));

    // Encoding
    let mut encoded_string = Vec::new(); // Note that this is LIFO, not FIFO

    let initial_state = L; // TODO: Is this general?

    println!("Initial state: {}", initial_state);

    let mut state = initial_state;

    for symbol in input.iter().rev() { // Encode in reverse order
        // Input symbol
        let symbol = *symbol;

        // Stream out bits from state until we're in the current symbol's precursor range (I -> Is)
        let precursor_range_end = match symbol {
            0 => max_state_0,
            _ => max_state_1,
        };

        while state > precursor_range_end {
            let output_bit = state & 0x01;
            encoded_string.push(output_bit);
            state >>= 1;
        }

        // Update state
        let next_state_index = state * ALPHABET_SIZE + symbol;
        state = encoding_table[next_state_index as usize];
    }

    let final_state = state;

    println!("Final state: {}", final_state);
    println!("Encoded string: {:?} ({} bits)", encoded_string, encoded_string.len());

    // Decoding
    let mut decoded_string = Vec::new();

    let mut state = final_state;

    for _ in 0..input.len() {
        // Pull in bits until state is in renormalization range (Is -> I, should mirror encoding output bits to move from I -> Is)
        while state < L {
            let input_bit = encoded_string.pop().unwrap();
            state <<= 1;
            state |= input_bit;
        }

        // Look up symbol and next state base in table
        let (symbol, next_state) = decoding_table[state as usize];

        // Output symbol
        decoded_string.push(symbol);

        // Update state
        state = next_state;
    }

    let final_state = state;

    println!("Final state: {}", final_state);
    println!("Decoded string: {:?} ({} bits)", decoded_string, decoded_string.len());
}
