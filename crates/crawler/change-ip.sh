
while true; do
    windscribe-cli disconnect

    LOC=$(windscribe-cli locations | grep '^    ' | shuf -n1 | xargs)

    echo "Connecting to: $LOC"

    windscribe-cli connect "$LOC"

    sleep 150
done
