#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Test configuration
UNIQUE_ID=$((RANDOM % 10000))
CUSTOM_ID="customid"
TEST_SERVER_HOST="127.0.0.1"
TEST_SERVER_PORT="8081"
TEST_DB_NAME="test_api_${UNIQUE_ID}.db"
TEST_DB_PATH="${TEST_DB_NAME}"

wait_for_server() {
  local max_attempts=50
  local wait_seconds=2
  local attempt=1
  local base_url="http://${TEST_SERVER_HOST}:${TEST_SERVER_PORT}"
  
  echo -e "${YELLOW}Waiting for server to be ready at ${base_url}...${NC}"
  
  while [ $attempt -le $max_attempts ]; do
    if curl -s --head --fail "${base_url}/health" >/dev/null 2>&1; then
      echo -e "${GREEN}Server is up and running after $(( attempt * wait_seconds )) seconds!${NC}"
      return 0
    fi
    
    echo -e "${YELLOW}Attempt ${attempt}/${max_attempts}: Server not ready yet, waiting ${wait_seconds}s...${NC}"
    sleep $wait_seconds
    attempt=$((attempt + 1))
  done
  
  echo -e "${RED}Server failed to start after $(( max_attempts * wait_seconds )) seconds!${NC}"
  return 1
}

setup() {
    echo -e "${YELLOW}Setting up test environment...${NC}"
    echo -e "${YELLOW}Current directory: $(pwd)${NC}"
    
    touch "${TEST_DB_PATH}"
    
    echo -e "${YELLOW}Starting server with test database at ${TEST_DB_PATH}...${NC}"
    echo -e "${YELLOW}Database URL: sqlite:${TEST_DB_PATH}${NC}"
    
    DATABASE_URL="sqlite:${TEST_DB_PATH}" \
    SERVER_HOST="${TEST_SERVER_HOST}" \
    SERVER_PORT="${TEST_SERVER_PORT}" \
    SHORT_URL_LENGTH="6" \
    LOG_LEVEL="info" \
    ENABLE_METRICS="false" \
    cargo run &
    SERVER_PID=$!
    
    # Check if server process is running
    if kill -0 $SERVER_PID 2>/dev/null; then
        echo -e "${GREEN}Server process started with PID: ${SERVER_PID}${NC}"
        # Wait for server to be ready to accept connections
        if wait_for_server; then
            echo -e "${GREEN}Server is ready to accept connections${NC}"
        else
            echo -e "${RED}Server didn't start properly${NC}"
            exit 1
        fi
    else
        echo -e "${RED}Failed to start server. Check logs for errors.${NC}"
        exit 1
    fi
}

teardown() {
    echo -e "${YELLOW}Tearing down test environment...${NC}"
    
    if [ -n "$SERVER_PID" ]; then
        if kill -0 $SERVER_PID 2>/dev/null; then
            echo -e "${YELLOW}Stopping server (PID: ${SERVER_PID})...${NC}"
            kill $SERVER_PID
            wait $SERVER_PID 2>/dev/null || true
        else
            echo -e "${YELLOW}Server already stopped.${NC}"
        fi
    fi
    
    if [ -f "${TEST_DB_PATH}" ]; then
        echo -e "${YELLOW}Removing test database: ${TEST_DB_PATH}${NC}"
        rm "${TEST_DB_PATH}"
    else
        echo -e "${YELLOW}Test database not found at: ${TEST_DB_PATH}${NC}"
    fi
    
    echo -e "${GREEN}Teardown complete.${NC}"
}

parse_json() {
    echo "$1" | jq -r "$2" 2>/dev/null || echo "null"
}

run_tests() {
    echo -e "${YELLOW}Running API tests...${NC}"
    
    local base_url="http://${TEST_SERVER_HOST}:${TEST_SERVER_PORT}"
    local test_failed=0
    
    echo -e "${YELLOW}Using test server at: ${base_url}${NC}"
    echo -e "${YELLOW}Using unique test database ID: ${UNIQUE_ID}${NC}"
    echo -e "${YELLOW}Using fixed custom URL ID: ${CUSTOM_ID}${NC}"
    
    # Test 1: Create a shortened URL
    echo -e "\n${YELLOW}Test 1: Creating a shortened URL...${NC}"
    RESPONSE=$(curl -X POST -H "Content-Type: application/json" -d '{"url": "https://example.com"}' ${base_url}/api/shorten -s -w "\nStatus: %{http_code}")
    
    STATUS_CODE=$(echo "$RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    RESPONSE_BODY=$(echo "$RESPONSE" | sed '$d')
    
    echo "$RESPONSE_BODY" | jq . || echo "$RESPONSE_BODY"
    
    if [ "$STATUS_CODE" -lt 200 ] || [ "$STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Request failed with status code: ${STATUS_CODE}${NC}"
        test_failed=1
        return $test_failed
    fi
    
    SHORT_ID=$(parse_json "$RESPONSE_BODY" ".short_id")
    if [ "$SHORT_ID" = "null" ]; then
        echo -e "${RED}Failed to extract short_id from response${NC}"
        echo -e "${RED}Response was: $RESPONSE_BODY${NC}"
        test_failed=1
        return $test_failed
    fi
    
    echo -e "${GREEN}Successfully created URL with short ID: $SHORT_ID${NC}"
    
    # Test 2: Create URL with custom ID
    echo -e "\n${YELLOW}Test 2: Creating a shortened URL with custom ID...${NC}"
    CUSTOM_RESPONSE=$(curl -X POST -H "Content-Type: application/json" -d "{\"url\": \"https://rust-lang.org\", \"custom_id\": \"$CUSTOM_ID\"}" ${base_url}/api/shorten -s -w "\nStatus: %{http_code}")
    
    CUSTOM_STATUS_CODE=$(echo "$CUSTOM_RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    CUSTOM_RESPONSE_BODY=$(echo "$CUSTOM_RESPONSE" | sed '$d')
    
    echo "$CUSTOM_RESPONSE_BODY" | jq . || echo "$CUSTOM_RESPONSE_BODY"
    
    if [ "$CUSTOM_STATUS_CODE" -lt 200 ] || [ "$CUSTOM_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Request failed with status code: ${CUSTOM_STATUS_CODE}${NC}"
        test_failed=1
        return $test_failed
    fi
    
    CUSTOM_SHORT_ID=$(parse_json "$CUSTOM_RESPONSE_BODY" ".short_id")
    if [ "$CUSTOM_SHORT_ID" = "null" ]; then
        echo -e "${RED}Failed to create URL with custom ID${NC}"
        echo -e "${RED}Response was: $CUSTOM_RESPONSE_BODY${NC}"
        test_failed=1
        return $test_failed
    fi
    
    echo -e "${GREEN}Successfully created URL with custom ID: $CUSTOM_ID${NC}"
    
    # Test 3: Get stats for first URL
    echo -e "\n${YELLOW}Test 3: Getting statistics for first URL...${NC}"
    STATS_RESPONSE=$(curl ${base_url}/api/stats/$SHORT_ID -s -w "\nStatus: %{http_code}")
    
    STATS_STATUS_CODE=$(echo "$STATS_RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    STATS_RESPONSE_BODY=$(echo "$STATS_RESPONSE" | sed '$d')
    
    echo "$STATS_RESPONSE_BODY" | jq . || echo "$STATS_RESPONSE_BODY"
    
    if [ "$STATS_STATUS_CODE" -lt 200 ] || [ "$STATS_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Stats request failed with status code: ${STATS_STATUS_CODE}${NC}"
        test_failed=1
        return $test_failed
    fi
    
    # Test 4: Get stats for custom URL
    echo -e "\n${YELLOW}Test 4: Getting statistics for custom URL...${NC}"
    CUSTOM_STATS_RESPONSE=$(curl ${base_url}/api/stats/$CUSTOM_ID -s -w "\nStatus: %{http_code}")
    
    CUSTOM_STATS_STATUS_CODE=$(echo "$CUSTOM_STATS_RESPONSE" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    CUSTOM_STATS_RESPONSE_BODY=$(echo "$CUSTOM_STATS_RESPONSE" | sed '$d')
    
    echo "$CUSTOM_STATS_RESPONSE_BODY" | jq . || echo "$CUSTOM_STATS_RESPONSE_BODY"
    
    if [ "$CUSTOM_STATS_STATUS_CODE" -lt 200 ] || [ "$CUSTOM_STATS_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Custom stats request failed with status code: ${CUSTOM_STATS_STATUS_CODE}${NC}"
        test_failed=1
        return $test_failed
    fi
    
    # Test 5: Test redirect
    echo -e "\n${YELLOW}Test 5: Calling redirection endpoint...${NC}"
    REDIRECT_RESPONSE=$(curl -I ${base_url}/$CUSTOM_ID -s)
    REDIRECT_STATUS=$(echo "$REDIRECT_RESPONSE" | head -n 1 | grep -o '[0-9]\{3\}')

    echo "$REDIRECT_RESPONSE" | head -n 3

    if ! [[ "$REDIRECT_STATUS" =~ ^3[0-9]{2}$ ]]; then
        echo -e "${RED}Redirect failed, expected 3xx status code, got: ${REDIRECT_STATUS}${NC}"
        test_failed=1
        return $test_failed
    fi

    echo -e "${GREEN}Redirect successful with status code: ${REDIRECT_STATUS}${NC}"

    # Test 6: Verify visit count
    echo -e "\n${YELLOW}Test 6: Verifying visit count increased...${NC}"
    UPDATED_STATS=$(curl ${base_url}/api/stats/$CUSTOM_ID -s -w "\nStatus: %{http_code}")

    UPDATED_STATS_STATUS_CODE=$(echo "$UPDATED_STATS" | grep -o 'Status: [0-9]*' | cut -d' ' -f2)
    UPDATED_STATS_BODY=$(echo "$UPDATED_STATS" | sed '$d')

    echo "$UPDATED_STATS_BODY" | jq . || echo "$UPDATED_STATS_BODY"

    if [ "$UPDATED_STATS_STATUS_CODE" -lt 200 ] || [ "$UPDATED_STATS_STATUS_CODE" -ge 300 ]; then
        echo -e "${RED}Updated stats request failed with status code: ${UPDATED_STATS_STATUS_CODE}${NC}"
        test_failed=1
        return $test_failed
    fi

    VISIT_COUNT=$(parse_json "$UPDATED_STATS_BODY" ".visit_count")

    if [ "$VISIT_COUNT" = "1" ]; then
        echo -e "${GREEN}Visit count test passed! Count = $VISIT_COUNT${NC}"
    else
        echo -e "${RED}Visit count test failed! Expected 1, got $VISIT_COUNT${NC}"
        echo -e "${RED}Response was: $UPDATED_STATS_BODY${NC}"
        test_failed=1
        return $test_failed
    fi
    
    # Test 7: List all URLs
    echo -e "\n${YELLOW}Test 7: Listing all URLs...${NC}"
    URLS_RESPONSE=$(curl ${base_url}/api/urls -s)
    echo "$URLS_RESPONSE" | jq . || echo "$URLS_RESPONSE"
    
    URL_COUNT=$(parse_json "$URLS_RESPONSE" ". | length")
    if [ "$URL_COUNT" -ge 2 ]; then
        echo -e "${GREEN}URL listing test passed! Found $URL_COUNT URLs${NC}"
    else
        echo -e "${RED}URL listing test failed! Expected at least 2 URLs, got $URL_COUNT${NC}"
        test_failed=1
        return $test_failed
    fi
    
    echo -e "\n${GREEN}All tests completed.${NC}"
    return $test_failed
}

# Main execution
trap teardown EXIT
setup
run_tests
TEST_RESULT=$?

echo -e "\n${GREEN}E2E tests completed.${NC}"
rm -f "${TEST_DB_PATH}" test_*.db 2>/dev/null || true
exit $TEST_RESULT