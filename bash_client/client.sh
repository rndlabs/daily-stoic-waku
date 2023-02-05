#!/bin/sh
# A bash client for polling the JSON-RPC interface of nwaku

# The URL of the JSON-RPC interface
URL="http://nwaku.nwaku.public.dappnode:8545"
PUBSUB_TOPIC="/waku/2/default-waku/proto"
CONTENT_TOPIC="/dailystoic/1/broadcast/proto"
REQUEST_CONTENT_TOPIC="/dailystoic/1/request/proto"

# A function that checks for the required dependencies
function checkDependencies() {
    # Check if jq is installed
    if ! [ -x "$(command -v jq)" ]; then
        log ERROR "jq is not installed."
        exit 1
    fi

    # Check if protoc is installed
    if ! [ -x "$(command -v protoc)" ]; then
        log ERROR "protoc is not installed."
        exit 1
    fi

    # Check if xxd is installed
    if ! [ -x "$(command -v xxd)" ]; then
        log ERROR "xxd is not installed."
        exit 1
    fi
}

# A function used for logging that prefixes the message with the current time and log level
function log() {
    echo "$(date +"%Y-%m-%d %H:%M:%S") $1 $2"
}

# A function that sets up the subscription
function setupSubscription() {
    METHOD="post_waku_v2_relay_v1_subscriptions" # The JSON-RPC method to call to subscribe to a `PubSub` topic.
    PARAMS='[["'"$PUBSUB_TOPIC"'"]]' # The JSON-RPC parameters is TOPIC as a string
    REQUEST='{"jsonrpc":"2.0","method":"'"$METHOD"'","params":'"$PARAMS"',"id":1}'

    # Inform the user that we are subscribing to the topic
    log INFO "Subscribing to the PubSub topic $PUBSUB_TOPIC and monitoring the content topic $CONTENT_TOPIC..."

    curl -s -X POST -H 'Content-Type: application/json' --data "$REQUEST" "$URL" > /dev/null
}

# A function that unsubscribes from a `PubSub` topic.
function unsubscribe() {
    METHOD="delete_waku_v2_relay_v1_subscriptions" # The JSON-RPC method to call to unsubscribe from a `PubSub` topic.
    PARAMS='[["'"$PUBSUB_TOPIC"'"]]' # The JSON-RPC parameters
    REQUEST='{"jsonrpc":"2.0","method":"'"$METHOD"'","params":'"$PARAMS"',"id":1}'

    curl -s -X DELETE -H 'Content-Type: application/json' --data "$REQUEST" "$URL" > /dev/null
}

# A function that makes the request for a daily stoic message
function requestDailyStoic() {
    METHOD="post_waku_v2_relay_v1_message" # The JSON-RPC method to call to publish a message

    # The `Request` protobuf requires a timestamp, get the current time in nano seconds
    NOW=$(date +%s%N)
    MESSAGE="timestamp: $NOW"

    # Encode the message with protoc using the `Request` type from the `dailystoic.proto` file
    PAYLOAD=$(echo $MESSAGE | protoc --encode=Request dailystoic.proto | xxd -p)

    # prefix payload with 0x
    # PAYLOAD="0x$PAYLOAD"

    PARAMS='["'"$PUBSUB_TOPIC"'",{"payload":"'"$PAYLOAD"'","contentTopic":"'"$REQUEST_CONTENT_TOPIC"'","timestamp":'"$NOW"'}]'
    REQUEST='{"jsonrpc":"2.0","method":"'"$METHOD"'","params":'"$PARAMS"',"id":2}'

    # Inform the user that we are requesting a daily stoic message
    log INFO "Requesting a daily stoic message..."

    # Send the request, we don't need the response
    curl -s -X POST -H 'Content-Type: application/json' --data "$REQUEST" "$URL" > /dev/null
}

#1. Check for the required dependencies
checkDependencies


# 2. Request a daily stoic message
requestDailyStoic

# 3. Poll the JSON-RPC interface for messages
while true; do
    METHOD="get_waku_v2_relay_v1_messages" # The JSON-RPC method to call to poll for messages
    PARAMS='["'"$PUBSUB_TOPIC"'"]' # The JSON-RPC parameters as an array consisting of the topic
    REQUEST='{"jsonrpc":"2.0","method":"'"$METHOD"'","params":'"$PARAMS"',"id":1}'

    # Send the request and store the response
    RESPONSE=$(curl -s -X GET -H 'Content-Type: application/json' --data "$REQUEST" "$URL")

    # If the response is not empty, print it
    if [ ! -z "$RESPONSE" ]; then
        # Use jq to filter the response where the response is an array of objects that have a 
        # `contentTopic` property that starts with `/dailystoic`
        RESPONSE=$(echo $RESPONSE | jq '.result | map(select(.contentTopic | startswith("/dailystoic/1/broadcast/proto")))')

        # Response is an array of objects, so we need to iterate over it
        # Each object has a `payload` property that is a base64 encoded string
        # We need to decode the string and print it
        for i in $(echo $RESPONSE | jq -r '.[] | @base64'); do
            # Get the uint8array payload
            PAYLOAD=$(echo $i | base64 --decode | jq -r '.payload')

            TEMP_PAYLOAD=""
            # Parse the uint8array
            for i in $(echo $PAYLOAD | jq -r '.[]'); do
                # append the value to the output with leading zeros
                TEMP_PAYLOAD+=$(printf "%02x" $i)
            done

            # Decode the protobuf message
            PAYLOAD=$(echo $TEMP_PAYLOAD | xxd -r -p | protoc --decode_raw)

            # Payload resembles something like the below:
            # 1: 1675537116 2: "Seneca" 3: "The greatest obstacle to living is expectancy, which hangs upon tomorrow and loses today. You are arranging what lies in Fortune\'s control, and abandoning what lies in yours. What are you looking at? To what goal are you straining? The whole future lies in uncertainty: live immediately."

            # Extract the timestamp
            TIMESTAMP=$(echo $PAYLOAD | grep -oP '1: \K([0-9]+)')
            # Convert the timestamp to a human readable date
            TIMESTAMP=$(date -d @$TIMESTAMP)
            # Extract the author between double quotes
            AUTHOR=$(echo $PAYLOAD | grep -oP '2: "\K([a-zA-Z\s]+)"' | tr -d '"' | sed 's/\\//g')
            # Extract the message which [:graph:] and [:space:]
            MESSAGE=$(echo $PAYLOAD | grep -oP '3: "\K([[:graph:][:space:]]+?)"' | tr -d '"' | sed 's/\\//g')

            # Print a new line
            echo

            # Pretty print the message, making the tags bold
            echo -e "\e[1mDate:\e[0m $TIMESTAMP"
            echo -e "\e[1mDaily Stoic:\e[0m \e[3m\e[1m$MESSAGE\e[0m - \e[1m\e[3m$AUTHOR\e[0m"

            # Print a new line
            echo
        done
    fi

    # Sleep for 1 second
    sleep 1
done
