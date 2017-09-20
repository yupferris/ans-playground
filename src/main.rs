fn main() {
    // Input symbols (unpacked bits for simplicity)
    let input = vec![0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    println!("Input: {:?} ({} bits)", input, input.len());

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

    // Scaled probability sum (determines # of states, coding precision)
    const M: u32 = 16;

    println!("M: {}", M);

    // Scaled integer probabilities (must sum to M)
    let f0_scaled = ((M as f64) * p0).round() as u32;
    let f1_scaled = M - f0_scaled;

    println!("f0 scaled: {}", f0_scaled);
    println!("f1 scaled: {}", f1_scaled);

    // Cumulative freq's (trivial in this case, but written out for practice)
    let b0 = 0;
    let b1 = f0_scaled;

    println!("b0: {}", b0);
    println!("b1: {}", b1);

    // Renormalization range
    println!("I = [{}, {}]", M, M * 2 - 1);

    // Precursor ranges
    println!("I0 = [{}, {}]", f0_scaled, f0_scaled * 2 - 1);
    println!("I1 = [{}, {}]", f1_scaled, f1_scaled * 2 - 1);

    // Encoding table/string (the simplest/most naive construction, not necessarily worst)
    let mut encoding_table = Vec::new();
    let mut encoding_string = Vec::new();

    for i in 0..M {
        encoding_table.push(M + i);
        encoding_string.push(if i < f0_scaled { 0 } else { 1 });
    }

    println!("Encoding table: {:?}", encoding_table);
    println!("Encoding string: {}", encoding_string.into_iter().fold(String::new(), |acc, x| format!("{}{}", acc, x)));

    // Encoding
    let mut encoded_string = Vec::new();

    let initial_state = M; // Only correct since we know the input string starts at 0

    println!("Initial state: {}", initial_state);

    let mut state = initial_state;

    for symbol in input.iter().rev() { // Note reversed input!!
        let symbol = *symbol;

        // Stream out bits from state until we're in the current symbol's precursor range
        let fs = match symbol {
            0 => f0_scaled,
            _ => f1_scaled,
        };
        let precursor_range_start = fs;
        let precursor_range_end = precursor_range_start * 2 - 1;

        while !(state >= precursor_range_start && state <= precursor_range_end) {
            let output_bit = state & 0x01;
            encoded_string.push(output_bit);
            state >>= 1;
        }

        let bs = match symbol {
            0 => b0,
            _ => b1,
        };
        let next_state_index = bs + (state - precursor_range_start);
        state = encoding_table[next_state_index as usize];
    }

    println!("Final state: {}", state);
    println!("Encoded string: {:?} ({} bits)", encoded_string, encoded_string.len());
}
