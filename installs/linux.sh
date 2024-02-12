find_bin_path() {
    for dir in /usr/local/bin /usr/bin /bin; do
        if echo "$PATH" | grep -qE "(^|:)$dir(:|$)"; then
            echo $dir
            return
        fi
    done
}

set -e

echo "Installing kill-tree..."

bin_path=$(find_bin_path)
if [ -z "$bin_path" ]; then
    echo "No bin path found in PATH"
    exit 1
fi

temp_dir=$(mktemp -d)
cd $temp_dir

echo "If required, please enter your password for sudo access..."

curl -L -s https://api.github.com/repos/oneofthezombies/kill-tree/releases/latest | \
    grep "kill-tree-linux-x86_64" | \
    grep "browser_download_url" | \
    cut -d '"' -f 4 | \
    xargs curl -L -s -o kill-tree && \
    chmod +x kill-tree && \
    mv -f kill-tree $bin_path/kill-tree && \
    rm -rf $temp_dir

echo "kill-tree install location: $bin_path/kill-tree"

echo "try printing version..."
kill-tree --version

echo "kill-tree installed successfully."
exit 0
