use std::env::args;
use std::fs::File;
use std::io::Read;
use std::iter;

const ALPHABET_SIZE: u32 = 16; // Only bits for now

// Scaled probability sum (determines # of states, coding precision)
const M: u32 = 128;

// Renormalization range
const K: u32 = 1; // Scaling factor (grows # of states, coding precision by a constant factor)
const L: u32 = M * K;

// State count
const STATE_COUNT: u32 = L * 2;

struct Context {
    encoding_table: Vec<u32>,
    decoding_table: Vec<(u32, u32)>,
    symbol_precursor_ranges: Vec<(u32, u32)>,
}

impl Context {
    fn new(symbol_frequencies: &[u32], state_index_start: Option<u32>, state_index_step: Option<u32>) -> Context {
        //println!("Symbol frequencies: {:?}", symbol_frequencies);

        let total_symbols = symbol_frequencies.iter().fold(0, |acc, x| acc + x);

        // Symbol probabilities (approx.)
        let symbol_probabilities =
            symbol_frequencies.iter()
            .map(|f| (*f as f64) / (total_symbols as f64))
            .collect::<Vec<_>>();

        //println!("Symbol probabilities: {:?}", symbol_probabilities);

        // Scaled integer probabilities (must sum to L)
        let mut scaled_symbol_probabilities =
            symbol_probabilities.iter()
            .map(|p| (*p * (L as f64)).ceil() as u32)
            .collect::<Vec<_>>();
        //  Due to rounding errors, we may have to adjust the scaled probabilities a bit so they sum to L.
        //  We do this by iteratively subtracting 1 from the probability that will suffer the least by doing so.
        while scaled_symbol_probabilities.iter().fold(0, |acc, x| acc + *x) > L {
            let adjust_index = scaled_symbol_probabilities.iter().enumerate().fold(None, |acc: Option<(usize, f64)>, (index, x)| {
                if *x <= 1 {
                    acc
                } else {
                    let adjusted_x = *x - 1;
                    let adjust_error = ((*x as f64) / (adjusted_x as f64)).log2() * (symbol_probabilities[index] as f64);
                    match acc {
                        Some(acc) => {
                            if adjust_error < acc.1 {
                                Some((index, adjust_error))
                            } else {
                                Some(acc)
                            }
                        }
                        _ => Some((index, adjust_error))
                    }
                }
            }).unwrap().0;
            scaled_symbol_probabilities[adjust_index] -= 1;
        }

        //println!("Scaled symbol probabilities: {:?}", scaled_symbol_probabilities);

        // Precursor ranges
        let symbol_precursor_ranges =
            scaled_symbol_probabilities.iter()
            .map(|p| (*p, *p * 2 - 1))
            .collect::<Vec<_>>();

        //println!("Symbol precursor ranges (Is, inclusive): {:?}", symbol_precursor_ranges);

        // Sorted symbols (the simplest/most naive construction, not necessarily worst, but likely to be due to precision loss)
        /*let sorted_symbols =
            scaled_symbol_probabilities.iter()
            .enumerate()
            .flat_map(|(s, p)| iter::repeat(s as u32).take(*p as _))
            .collect::<Vec<_>>();*/

        let sorted_symbols = {
            const UNALLOCATED: u32 = ALPHABET_SIZE;
            let mut ret = vec![UNALLOCATED; L as usize];
            let mut target_index = state_index_start.unwrap_or(0);
            for (symbol, probability) in scaled_symbol_probabilities.iter().enumerate() {
                for _ in 0..*probability {
                    let mut temp_target_index = target_index;
                    while ret[temp_target_index as usize] != UNALLOCATED {
                        temp_target_index = (temp_target_index + 1) & (L - 1);
                    }
                    ret[temp_target_index as usize] = symbol as _;

                    target_index = (target_index + state_index_step.unwrap_or(49)) & (L - 1);
                }
            }
            ret
        };

        //println!("Sorted symbols: {}", sorted_symbols.iter().fold(String::new(), |acc, x| format!("{}{:1x}", acc, *x)));

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

        //println!("Encoding table: {:?}", encoding_table);
        //println!("Decoding table: {:?}", decoding_table);

        Context {
            encoding_table: encoding_table,
            decoding_table: decoding_table,
            symbol_precursor_ranges: symbol_precursor_ranges,
        }
    }
}

fn main() {
    let input_file_name = args().nth(1).expect("Couldn't read input file name arg");

    // Input symbols
    //let input = vec![0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // Unpacked, raw symbols
    let input = {
        let mut file = File::open(input_file_name).expect("Couldn't open input file");
        let mut ret = Vec::new();
        file.read_to_end(&mut ret).expect("Couldn't read input file bytes");
        ret.into_iter().map(|x| x as u32).collect::<Vec<_>>()
    };

    println!("Alphabet size: {}", ALPHABET_SIZE);
    //println!("Input string: {:?} ({} bits)", input, input.len() * 8);
    println!("Input size: {} bits ({} bytes)", input.len() * 8, input.len());

    println!("M: {}", M);

    println!("K: {}", K);
    println!("L: {}", L);
    println!("Renormalization range (I) = [{}, {}]", L, L * 2 - 1);

    println!("State count: {}", STATE_COUNT);

    let mut first_nybble_symbol_frequencies = vec![0; 16];
    let mut second_nybble_symbol_frequencies = vec![vec![0; 16]; 16];

    for byte in input.iter() {
        let byte = *byte;

        let first_nybble = byte >> 4;
        let second_nybble = byte & 0x0f;

        first_nybble_symbol_frequencies[first_nybble as usize] += 1;
        second_nybble_symbol_frequencies[first_nybble as usize][second_nybble as usize] += 1;
    }

    let mut best_size_bits = None;

    for state_index_start in 0..128 {
        for state_index_step in 0..128 {
            let first_nybble_context = Context::new(&first_nybble_symbol_frequencies, Some(state_index_start), Some(state_index_step));
            let second_nybble_contexts = second_nybble_symbol_frequencies.iter().map(|f| Context::new(f, Some(state_index_start), Some(state_index_step))).collect::<Vec<_>>();

            // Encoding
            let mut encoded_string = Vec::new(); // Note that this is LIFO, not FIFO

            let mut state = L;

            //println!("Initial encoding state (x): {}", state);

            for symbol in input.iter().rev() { // Encode in reverse order
                // Input symbol
                let symbol = *symbol;

                let first_nybble = symbol >> 4;
                let second_nybble = symbol & 0x0f;

                state = encode(second_nybble, state, &second_nybble_contexts[first_nybble as usize], &mut encoded_string);
                state = encode(first_nybble, state, &first_nybble_context, &mut encoded_string);
            }

            let final_state = state;

            //println!("Final encoding state (x'): {}", final_state);
            //println!("Encoded string: {:?} ({} bits)", encoded_string, encoded_string.len());
            let ratio = (1.0 - (encoded_string.len() as f64) / ((input.len() * 8) as f64)) * 100.0;
            best_size_bits = match best_size_bits {
                Some(current_best_size_bits) => {
                    if encoded_string.len() < current_best_size_bits {
                        println!("New best: {} bits (~{} bytes), start: {}, step {}", encoded_string.len(), encoded_string.len() / 8, state_index_start, state_index_step);
                        println!("Encoded string size: {} bits (~{} bytes, {:.*}%)", encoded_string.len(), encoded_string.len() / 8, 2, ratio);
                        Some(encoded_string.len())
                    } else {
                        Some(current_best_size_bits)
                    }
                }
                _ => {
                    println!("New best: {} bits (~{} bytes), start: {}, step {}", encoded_string.len(), encoded_string.len() / 8, state_index_start, state_index_step);
                    println!("Encoded string size: {} bits (~{} bytes, {:.*}%)", encoded_string.len(), encoded_string.len() / 8, 2, ratio);
                    Some(encoded_string.len())
                }
            };

            // Decoding
            let mut decoded_string = Vec::new();

            let mut state = final_state;

            //println!("Initial decoding state (x): {}", state);

            for _ in 0..input.len() {
                let (first_nybble, new_state) = decode(state, &first_nybble_context, &mut encoded_string);
                state = new_state;
                let (second_nybble, new_state) = decode(state, &second_nybble_contexts[first_nybble as usize], &mut encoded_string);
                state = new_state;

                let symbol = (first_nybble << 4) | second_nybble;

                // Output symbol
                decoded_string.push(symbol);
            }

            let final_state = state;

            //println!("Final decoding state (x'): {}", final_state);
            //println!("Decoded string: {:?} ({} bits)", decoded_string, decoded_string.len() * 8);
            //println!("Decoded string size: {} bits ({} bytes)", decoded_string.len() * 8, decoded_string.len());

            if decoded_string == input {
                //println!("Input/output match!");
            } else {
                println!("Input/output don't match :(");
            }
        }
    }
}

fn encode(symbol: u32, mut state: u32, context: &Context, encoded_string: &mut Vec<u32>) -> u32 {
    // Stream out bits from state until we're in the current symbol's precursor range (I -> Is)
    let precursor_range_end = context.symbol_precursor_ranges[symbol as usize].1;

    while state > precursor_range_end {
        let output_bit = state & 0x01;
        encoded_string.push(output_bit);
        state >>= 1;
    }

    // Update state
    let next_state_index = state * ALPHABET_SIZE + symbol;
    state = context.encoding_table[next_state_index as usize];

    state
}

fn decode(mut state: u32, context: &Context, encoded_string: &mut Vec<u32>) -> (u32, u32) {
    // Look up symbol and next state base in table
    let (symbol, next_state_base) = context.decoding_table[state as usize];

    // Update state
    state = next_state_base;

    // Pull in bits until state is in renormalization range (Is -> I, should mirror encoding output bits to move from I -> Is)
    while state < L {
        let input_bit = encoded_string.pop().unwrap();
        state <<= 1;
        state |= input_bit;
    }

    (symbol, state)
}
