# This file is for local debugging purposes only.
# To prepare your environment, run the following commands:
# 1. Run `mkdir data{,-out,-backup}`
# 2. Place your images (~15-20) in `data-backup/`
# 3. Run `just reset` or `just run`

# Resets the environment
reset:
    rm -r data{,-out}
    cp -r data{-backup,}
    mkdir data-out

# Runs the program after resetting the environment
run: reset
    cargo run --release -- data/ data-out/
