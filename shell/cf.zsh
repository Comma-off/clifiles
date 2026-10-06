# clifiles shell integration for bash/zsh.
# Source this from .bashrc / .zshrc, then run `cf` to browse and `cd` on exit.
cf() {
    local cwd_file
    cwd_file="$(mktemp)"
    command clifiles --cwd-file "$cwd_file" "$@"
    if [ -s "$cwd_file" ]; then
        cd -- "$(cat "$cwd_file")" || true
    fi
    rm -f "$cwd_file"
}
