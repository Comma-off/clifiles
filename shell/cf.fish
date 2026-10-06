# clifiles shell integration for fish.
# Source this from config.fish, then run `cf` to browse and `cd` on exit.
function cf
    set -l cwd_file (mktemp)
    command clifiles --cwd-file "$cwd_file" $argv
    if test -s "$cwd_file"
        cd (cat "$cwd_file")
    end
    rm -f "$cwd_file"
end
