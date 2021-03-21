# payments-engine

All is idiomatic Rust. Just safe code.
In order to not read in the entire file at ones I used BufReader reading one line at a time.
Every line creates a Transaction instance and the memory for the line is freed at the same time
