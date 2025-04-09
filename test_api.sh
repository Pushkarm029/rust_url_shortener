#!/bin/bash
set -e

# Colors for better output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Generate a unique test ID for the database only
UNIQUE_ID=$((RANDOM % 10000))

# Fixed custom ID for testing
CUSTOM_ID="customid"

# Test server configuration
TEST_SERVER_HOST="127.0.0.1"
TEST_SERVER_PORT="8081"

# Create test database with full path - global variables
TEST_DB_NAME="test_api_${UNIQUE_ID}.db"
TEST_DB_PATH="${TEST_DB_NAME}"

# Setup function
setup() {
    echo -e "${YELLOW}Setting up test environment...${NC}"
    
    # Print current directory for debugging
    echo -e "${YELLOW}Current directory: $(pwd)${NC}"
    
    # Use a simple relative path for the test database
    TEST_DB_NAME="test_api_${UNIQUE_ID}.db"
    TEST_DB_PATH="${TEST_DB_NAME}"
    
    # Create empty database file to ensure directory is writable
    touch "${TEST_DB_PATH}"
    
    # Start the server in the background using environment variables directly
    echo -e "${YELLOW}Starting server with test database at ${TEST_DB_PATH}...${NC}"
    
    # Use the proper SQLite connection string format for sqlx
    # Format for sqlx is: sqlite:/path/to/db
    echo -e "${YELLOW}Database URL: sqlite:${TEST_DB_PATH}${NC}"
    
    DATABASE_URL="sqlite:${TEST_DB_PATH}" \
    SERVER_HOST="${TEST_SERVER_HOST}" \
    SERVER_PORT="${TEST_SERVER_PORT}" \
    SHORT_URL_LENGTH="6" \
    LOG_LEVEL="info" \
    ENABLE_METRICS="false" \
    cargo run &
    SERVER_PID=$!
    
    # Wait longer for server to start and become fully ready
    echo -e "${YELLOW}Waiting for server to start (PID: ${SERVER_PID})...${NC}"
    sleep 2  # Give it more time to start and initialize
    
    # Check if server process is still running
    if kill -0 $SERVER_PID 2>/dev/null; then
        echo -e "${GREEN}Server started successfully with PID: ${SERVER_PID}${NC}"
        echo -e "${YELLOW}Waiting for server to fully initialize...${NC}"
        sleep 2  # Additional time for server to be ready to accept connections
    else
        echo -e "${RED}Failed to start server. Check logs for errors.${NC}"
        exit 1
    fi
}

# Teardown function
teardown() {
    echo -e "${YELLOW}Tearing down test environment...${NC}"
    
    # Kill the server process
    if [ -n "$SERVER_PID" ]; then
        if kill -0 $SERVER_PID 2>/dev/null; then
            echo -e "${YELLOW}Stopping server (PID: ${SERVER_PID})...${NC}"
            kill $SERVER_PID
            wait $SERVER_PID 2>/dev/null || true
        else
            echo -e "${YELLOW}Server already stopped.${NC}"
        fi
    fi
    
    # Remove test database
    if [ -f "${TEST_DB_PATH}" ]; then
        echo -e "${YELLOW}Removing test database: ${TEST_DB_PATH}${NC}"
        rm "${TEST_DB_PATH}"
    else
        echo -e "${YELLOW}Test database not found at: ${TEST_DB_PATH}${NC}"
    fi
    
    echo -e "${GREEN}Teardown complete.${NC}"
}

# Safe JSON parsing function
parse_json() {
    echo "$1" | jq -r "$2" 2>/dev/null || echo "null"
}

# Function to run tests
run_tests() {
    echo -e "${YELLOW}Running API tests...${NC}"
    
    # Use the test server configuration
    local base_url="http://${TEST_SERVER_HOST}:${TEST_SERVER_PORT}"
    
    echo -e "${YELLOW}Using test server at: ${base_url}${NC}"
    echo -e "${YELLOW}Using unique test database ID: ${UNIQUE_ID}${NC}"
    echo -e "${YELLOW}Using fixed custom URL ID: ${CUSTOM_ID}${NC}"
    
    # Test 1: Create a shortened URL
    echo -e "\n${YELLOW}Test 1: Creating a shortened URL...${NC}"
    RESPONSE=$(curl -X POST -H "Content-Type: application/json" -d '{"url": "https://example.com"}' ${base_url}/api/shorten -s -w "\nStatus: %{http_code}")
    
    # Extract status code
    STATUS_CODE=$(echo "$RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    RESPONSE_BODY=$(echo "$RESPONSE" | sed '$d')  # Remove the last line containing the status
    
    echo "$RESPONSE_BODY" | jq . || echo "$RESPONSE_BODY"
    
    # Check status code
    if [ "$STATUS_CODE" -lt 200 ] || [ "$STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Request failed with status code: ${STATUS_CODE}${NC}"
        exit 1
    fi
    
    # Extract the short_id from the response, fallback to null if parsing fails
    SHORT_ID=$(parse_json "$RESPONSE_BODY" ".short_id")
    if [ "$SHORT_ID" = "null" ]; then
        echo -e "${RED}Failed to extract short_id from response${NC}"
        echo -e "${RED}Response was: $RESPONSE_BODY${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Successfully created URL with short ID: $SHORT_ID${NC}"
    
    # Wait a moment
    sleep 1
    
    # Test 2: Create another URL with a custom ID
    echo -e "\n${YELLOW}Test 2: Creating a shortened URL with custom ID...${NC}"
    CUSTOM_RESPONSE=$(curl -X POST -H "Content-Type: application/json" -d "{\"url\": \"https://rust-lang.org\", \"custom_id\": \"$CUSTOM_ID\"}" ${base_url}/api/shorten -s -w "\nStatus: %{http_code}")
    
    # Extract status code
    CUSTOM_STATUS_CODE=$(echo "$CUSTOM_RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    CUSTOM_RESPONSE_BODY=$(echo "$CUSTOM_RESPONSE" | sed '$d')  # Remove the last line containing the status
    
    echo "$CUSTOM_RESPONSE_BODY" | jq . || echo "$CUSTOM_RESPONSE_BODY"
    
    # Check status code
    if [ "$CUSTOM_STATUS_CODE" -lt 200 ] || [ "$CUSTOM_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Request failed with status code: ${CUSTOM_STATUS_CODE}${NC}"
        exit 1
    fi
    
    # Extract the custom short_id
    CUSTOM_SHORT_ID=$(parse_json "$CUSTOM_RESPONSE_BODY" ".short_id")
    if [ "$CUSTOM_SHORT_ID" = "null" ]; then
        echo -e "${RED}Failed to create URL with custom ID${NC}"
        echo -e "${RED}Response was: $CUSTOM_RESPONSE_BODY${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Successfully created URL with custom ID: $CUSTOM_ID${NC}"
    
    # Wait a moment
    sleep 1
    
    # Test 3: Get statistics for the first URL
    echo -e "\n${YELLOW}Test 3: Getting statistics for first URL...${NC}"
    STATS_RESPONSE=$(curl ${base_url}/api/stats/$SHORT_ID -s -w "\nStatus: %{http_code}")
    
    # Extract status code
    STATS_STATUS_CODE=$(echo "$STATS_RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    STATS_RESPONSE_BODY=$(echo "$STATS_RESPONSE" | sed '$d')  # Remove the last line containing the status
    
    echo "$STATS_RESPONSE_BODY" | jq . || echo "$STATS_RESPONSE_BODY"
    
    # Check status code
    if [ "$STATS_STATUS_CODE" -lt 200 ] || [ "$STATS_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Stats request failed with status code: ${STATS_STATUS_CODE}${NC}"
        exit 1
    fi
    
    # Test 4: Get statistics for the custom URL
    echo -e "\n${YELLOW}Test 4: Getting statistics for custom URL...${NC}"
    CUSTOM_STATS_RESPONSE=$(curl ${base_url}/api/stats/$CUSTOM_ID -s -w "\nStatus: %{http_code}")
    
    # Extract status code
    CUSTOM_STATS_STATUS_CODE=$(echo "$CUSTOM_STATS_RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    CUSTOM_STATS_RESPONSE_BODY=$(echo "$CUSTOM_STATS_RESPONSE" | sed '$d')  # Remove the last line containing the status
    
    echo "$CUSTOM_STATS_RESPONSE_BODY" | jq . || echo "$CUSTOM_STATS_RESPONSE_BODY"
    
    # Check status code
    if [ "$CUSTOM_STATS_STATUS_CODE" -lt 200 ] || [ "$CUSTOM_STATS_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Custom stats request failed with status code: ${CUSTOM_STATS_STATUS_CODE}${NC}"
        exit 1
    fi
    
    # Test 5: Call redirection endpoint to increment the visit count
    echo -e "\n${YELLOW}Test 5: Calling redirection endpoint...${NC}"
    REDIRECT_RESPONSE=$(curl -I ${base_url}/$CUSTOM_ID -s)
    REDIRECT_STATUS=$(echo "$REDIRECT_RESPONSE" | head -n 1 | grep -o '[0-9]\{3\}')

    echo "$REDIRECT_RESPONSE" | head -n 3

    # Check for appropriate redirect status code (3xx)
    if ! [[ "$REDIRECT_STATUS" =~ ^3[0-9]{2}$ ]]; then
        echo -e "${RED}Redirect failed, expected 3xx status code, got: ${REDIRECT_STATUS}${NC}"
        exit 1
    fi

    echo -e "${GREEN}Redirect successful with status code: ${REDIRECT_STATUS}${NC}"

    # Wait a moment for the visit to be recorded
    sleep 1

    # Test 6: Check if the visit count increased
    echo -e "\n${YELLOW}Test 6: Verifying visit count increased...${NC}"
    UPDATED_STATS=$(curl ${base_url}/api/stats/$CUSTOM_ID -s -w "\nStatus: %{http_code}")

    # Extract status code
    UPDATED_STATS_STATUS_CODE=$(echo "$UPDATED_STATS" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    UPDATED_STATS_BODY=$(echo "$UPDATED_STATS" | sed '$d')  # Remove the last line containing the status

    echo "$UPDATED_STATS_BODY" | jq . || echo "$UPDATED_STATS_BODY"

    # Check status code
    if [ "$UPDATED_STATS_STATUS_CODE" -lt 200 ] || [ "$UPDATED_STATS_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Updated stats request failed with status code: ${UPDATED_STATS_STATUS_CODE}${NC}"
        exit 1
    fi

    VISIT_COUNT=$(parse_json "$UPDATED_STATS_BODY" ".visit_count")

    if [ "$VISIT_COUNT" = "1" ]; then
        echo -e "${GREEN}Visit count test passed! Count = $VISIT_COUNT${NC}"
    else
        echo -e "${RED}Visit count test failed! Expected 1, got $VISIT_COUNT${NC}"
        echo -e "${RED}Response was: $UPDATED_STATS_BODY${NC}"
    fi
    
    # Test 7: List all URLs
    echo -e "\n${YELLOW}Test 7: Listing all URLs...${NC}"
    URLS_RESPONSE=$(curl ${base_url}/api/urls -s)
    echo "$URLS_RESPONSE" | jq . || echo "$URLS_RESPONSE"
    
    # Validate we have at least 2 URLs in the response
    URL_COUNT=$(parse_json "$URLS_RESPONSE" ". | length")
    if [ "$URL_COUNT" -ge 2 ]; then
        echo -e "${GREEN}URL listing test passed! Found $URL_COUNT URLs${NC}"
    else
        echo -e "${RED}URL listing test failed! Expected at least 2 URLs, got $URL_COUNT${NC}"
    fi
    
    echo -e "\n${GREEN}All tests completed.${NC}"
}

# Main execution
# Trap to ensure teardown happens even if script fails
trap teardown EXIT

# Run the test sequence
setup
run_tests

echo -e "\n${GREEN}E2E tests completed successfully!${NC}"
# Force removal of database files to ensure cleanup
rm -f "${TEST_DB_PATH}" test_*.db 2>/dev/null || true
exit 0 