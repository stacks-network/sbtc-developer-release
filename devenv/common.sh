show_help() {
    echo "Usage: $0 [-h|--help] <profile>"
    echo "Available profiles: min (bitcoin, stacks, electrs), full"
    echo "Configs and runtimes may change depending on the profile."
    exit 0
}

docker_runner() {
    if [ "$profile" == "min" ]; then
        docker compose -f min-docker-compose.yml $1 $2
    elif [ "$profile" == "full" ]; then
        docker compose $1 $2
    else
        echo "Invalid profile. Available profiles: min, full"
    fi
}

run() {
    local profile=""
    while [[ $# -gt 0 ]]; do
        key="$1"

        case $key in
        -h | --help)
            show_help
            ;;
        min | full)
            if ! [ -z "$profile" ]; then
                echo "Profile '$profile' already set"
                show_help
                exit 1
            fi
            profile="$1"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
        esac
    done

    if [ -z "$profile" ]; then
        echo "You must specify a profile (min or full)"
        show_help
        exit 1
    fi

    local subcommand=$(basename $0 .sh)
    docker_runner $subcommand "$FLAGS"
}
