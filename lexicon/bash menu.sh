#!/bin/bash

# wrote by alexander14k

base_name=$(basename "$PWD")
os=$(uname -s)

if [[ "$os" == "Linux" ]]; then
    source $HOME/.cargo/env
    app_name="$base_name"
elif [[ "$os" == "Darwin" ]]; then
    source $HOME/.cargo/env
    app_name="$base_name"
else
    app_name="$base_name.exe"
fi

show_menu() {
    output_menu="output \"$base_name\"

    c = clean target
    m = build target
        s = show target details
        rc = run target in client mode
        rs = run target in server mode
    x = exit
"
    echo "$output_menu"
}

run_menu() {
    show_menu
    while true; do
        read -p "enter: " choice
        if [[ "$choice" = "c" ]]; then
            cargo clean
        elif [[ "$choice" = "m" ]]; then
            cargo build --release
            if [[ "$?" -eq "0" ]]; then
                if [[ ! -d "test" ]]; then
                    mkdir "test"
                fi
                cp "target/release/$app_name" "test"
            fi
        elif [[ "$choice" = "s" ]]; then
            if [[ -f "target/release/$app_name" ]]; then
                size_bytes=$(stat -c %s "target/release/$app_name")
                size_kb=$((size_bytes / 1024))
                echo "bin size: ${size_kb} kB"
                echo "bin md5sum: " $(md5sum "target/release/$app_name")
            fi
        elif [[ "$choice" = "rc" ]]; then
            ./"test"/$app_name
        elif [[ "$choice" = "rs" ]]; then
            ./"test"/$app_name -s &
        elif [[ "$choice" = "x" ]]; then
            break
        else
            show_menu
        fi
    done
}

warning() {
    echo "if server is started and window is blocked
    use Ctrl+C to stop it"
}

run_menu
warning