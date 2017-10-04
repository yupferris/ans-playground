use std::iter;

fn main() {
    // Input symbols
    const ALPHABET_SIZE: u32 = 2; // Only bits for now
    let input = vec![0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // Unpacked, raw symbols

    println!("Alphabet size: {}", ALPHABET_SIZE);
    println!("Input string: {:?} ({} bits)", input, input.len());

    // Symbol frequencies
    let symbol_frequencies =
        (0..ALPHABET_SIZE)
        .map(|s| input.iter().fold(0, |acc, x| if *x == s { acc + 1 } else { acc }))
        .collect::<Vec<_>>();

    println!("Symbol frequencies: {:?}", symbol_frequencies);

    // Symbol probabilities (approx.)
    let symbol_probabilities =
        symbol_frequencies.iter()
        .map(|f| (*f as f64) / (input.len() as f64))
        .collect::<Vec<_>>();

    println!("Symbol probabilities: {:?}", symbol_probabilities);

    // Scaled probability sum (determines # of states, coding precision)
    const M: u32 = 16;

    println!("M: {}", M);

    // Renormalization range
    const K: u32 = 1; // Scaling factor (grows # of states, coding precision by a constant factor)
    const L: u32 = M * K;

    println!("L: {}", L);
    println!("Renormalization range (I) = [{}, {}]", L, L * 2 - 1);

    // Scaled integer probabilities (must sum to L)
    //  Note that in very extreme cases (eg. a symbol has ~0 probability) this algorithm might fail. :)
    let mut scaled_symbol_probabilities =
        symbol_probabilities.iter()
        .map(|p| {
            let mut ret = (*p * (L as f64)).floor() as u32;
            if ret < 1 {
                ret = 1;
            }
            ret
        })
        .collect::<Vec<_>>();
    //  Due to rounding errors, we may have to adjust the scaled probabilities a bit so they sum to L.
    //  Adjusting the last prob is the easiest way to do this, although it's likely not very accurate.
    while scaled_symbol_probabilities.iter().fold(0, |acc, x| acc + *x) < L {
        let i = scaled_symbol_probabilities.len() - 1;
        scaled_symbol_probabilities[i] += 1;
    }

    println!("Scaled symbol probabilities: {:?}", scaled_symbol_probabilities);

    // Cumulative scaled probalities
    let cumulative_scaled_symbol_probabilities =
        scaled_symbol_probabilities.iter()
        .scan(0, |acc, x| {
            let ret = *acc;

            *acc += *x;

            Some(ret)
        })
        .collect::<Vec<_>>();

    println!("Cumulative scaled symbol probabilities: {:?}", cumulative_scaled_symbol_probabilities);

    // Precursor ranges
    let symbol_precursor_ranges =
        scaled_symbol_probabilities.iter()
        .map(|p| (*p, *p * 2 - 1))
        .collect::<Vec<_>>();

    println!("Symbol precursor ranges (Is, inclusive): {:?}", symbol_precursor_ranges);

    // Sorted symbols (the simplest/most naive construction, not necessarily worst, but likely to be due to precision loss)
    let sorted_symbols =
        scaled_symbol_probabilities.iter()
        .enumerate()
        .flat_map(|(s, p)| iter::repeat(s as u32).take(*p as _))
        .collect::<Vec<_>>();

    println!("Sorted symbols: {}", sorted_symbols.iter().fold(String::new(), |acc, x| format!("{}{}", acc, *x)));

    // State count
    const STATE_COUNT: u32 = L * 2;

    println!("State count: {}", STATE_COUNT);

    // Encoding/decoding table/string (the simplest/most naive construction as sparse 2D arrays, basically the worst possible option but most intuitive)
    let mut encoding_table = vec![0; (STATE_COUNT * ALPHABET_SIZE) as usize];
    let mut decoding_table = vec![(0, 0); STATE_COUNT as usize];

    let mut symbol_max_states =
        scaled_symbol_probabilities.iter()
        .map(|p| *p - 1) // - 1 because we want this to be an inclusive upper bound
        .collect::<Vec<_>>();

    for to_state in L..L * 2 {
        let symbol = sorted_symbols[(to_state - L) as usize];

        symbol_max_states[symbol as usize] += 1;
        let from_state = symbol_max_states[symbol as usize];

        encoding_table[(from_state * ALPHABET_SIZE + symbol) as usize] = to_state;

        decoding_table[to_state as usize] = (symbol, from_state);
    }

    println!("Encoding table: {:?}", encoding_table);
    println!("Decoding table: {:?}", decoding_table);

    // Encoding
    let mut encoded_string = Vec::new(); // Note that this is LIFO, not FIFO

    let mut state = L;

    println!("Initial encoding state (x): {}", state);

    for symbol in input.iter().rev() { // Encode in reverse order
        // Input symbol
        let symbol = *symbol;

        // Stream out bits from state until we're in the current symbol's precursor range (I -> Is)
        let precursor_range_end = symbol_precursor_ranges[symbol as usize].1;

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

    println!("Final encoding state (x'): {}", final_state);
    println!("Encoded string: {:?} ({} bits)", encoded_string, encoded_string.len());

    // Decoding
    let mut decoded_string = Vec::new();

    let mut state = final_state;

    println!("Initial decoding state (x): {}", state);

    for _ in 0..input.len() {
        // Look up symbol and next state base in table
        let (symbol, next_state_base) = decoding_table[state as usize];

        // Output symbol
        decoded_string.push(symbol);

        // Update state
        state = next_state_base;

        // Pull in bits until state is in renormalization range (Is -> I, should mirror encoding output bits to move from I -> Is)
        while state < L {
            let input_bit = encoded_string.pop().unwrap();
            state <<= 1;
            state |= input_bit;
        }
    }

    let final_state = state;

    println!("Final decoding state (x'): {}", final_state);
    println!("Decoded string: {:?} ({} bits)", decoded_string, decoded_string.len());
}
