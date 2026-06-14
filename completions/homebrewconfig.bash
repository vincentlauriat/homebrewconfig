# bash completion for homebrewconfig
_homebrewconfig() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="--profile --set --unset --import-preset --export-preset --apply --dry-run --list --json --help --version"

    # Flags that take a file path argument.
    case "$prev" in
        -p|--profile|--import-preset|--export-preset)
            COMPREPLY=( $(compgen -f -- "$cur") )
            return 0
            ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
        return 0
    fi
}
complete -F _homebrewconfig homebrewconfig
