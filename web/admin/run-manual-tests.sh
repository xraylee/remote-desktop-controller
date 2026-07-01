#!/bin/bash
# RDCS Web Admin - Manual Test Execution Script
# Usage: ./run-manual-tests.sh

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

API_URL="http://localhost:8080"
WEB_URL="http://localhost:3000"

PASS=0
FAIL=0

pass() {
  echo -e "${GREEN}  ✅ PASS${NC}: $1"
  ((PASS++))
}

fail() {
  echo -e "${RED}  ❌ FAIL${NC}: $1"
  echo -e "     ${YELLOW}$2${NC}"
  ((FAIL++))
}

section() {
  echo -e "\n${BLUE}━━━ $1 ━━━${NC}"
}

echo -e "${BLUE}╔═══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  RDCS Web Admin - Test Execution         ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════╝${NC}"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
section "Pre-Test: Environment Check"
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo -n "Checking API service... "
if curl -s "$API_URL/healthz" > /dev/null 2>&1; then
  pass "API service running ($API_URL)"
else
  fail "API service not running" "Start with: cd services/api && go run cmd/api/main.go"
  exit 1
fi

echo -n "Checking Web UI... "
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$WEB_URL" 2>&1)
if [ "$HTTP_CODE" = "200" ]; then
  pass "Web UI running ($WEB_URL)"
else
  fail "Web UI not running" "Start with: cd web/admin && npm run dev"
  exit 1
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
section "Automated API Tests"
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Test 1: Successful login
echo -e "\n▶ Test 1: Successful Login"
RESP=$(curl -s -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@rdcs-test.local","password":"test123"}')

if echo "$RESP" | grep -q "access_token"; then
  pass "Admin login returns access_token"
  TOKEN=$(echo "$RESP" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)

  if echo "$RESP" | grep -q "refresh_token"; then
    pass "Login returns refresh_token"
  else
    fail "Login missing refresh_token" "$RESP"
  fi

  if echo "$RESP" | grep -q "member"; then
    pass "Login returns member info"
  else
    fail "Login missing member info" "$RESP"
  fi
else
  fail "Login failed" "$RESP"
  TOKEN=""
fi

# Test 2: Invalid credentials
echo -e "\n▶ Test 2: Invalid Credentials"
RESP=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@rdcs-test.local","password":"wrongpassword"}')
STATUS=$(echo "$RESP" | tail -n 1)

if [ "$STATUS" = "401" ]; then
  pass "Wrong password returns 401"
else
  fail "Wrong password should return 401" "Got $STATUS"
fi

# Test 3: Non-existent user
echo -e "\n▶ Test 3: Non-existent User"
RESP=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"nobody@example.com","password":"test123"}')
STATUS=$(echo "$RESP" | tail -n 1)

if [ "$STATUS" = "401" ]; then
  pass "Non-existent user returns 401"
else
  fail "Non-existent user should return 401" "Got $STATUS"
fi

# Test 4: Protected route without token
echo -e "\n▶ Test 4: Protected Route (No Auth)"
RESP=$(curl -s -w "\n%{http_code}" -X GET "$API_URL/api/v1/teams/a0000000-0000-0000-0000-000000000001/devices")
STATUS=$(echo "$RESP" | tail -n 1)

if [ "$STATUS" = "401" ]; then
  pass "Protected route returns 401 without token"
else
  fail "Protected route should return 401" "Got $STATUS"
fi

# Test 5: Protected route with valid token
if [ -n "$TOKEN" ]; then
  echo -e "\n▶ Test 5: Protected Route (With Auth)"
  RESP=$(curl -s -w "\n%{http_code}" -X GET "$API_URL/api/v1/teams/a0000000-0000-0000-0000-000000000001/devices" \
    -H "Authorization: Bearer $TOKEN")
  STATUS=$(echo "$RESP" | tail -n 1)

  if [ "$STATUS" = "200" ]; then
    pass "Protected route returns 200 with valid token"
  else
    fail "Protected route should return 200" "Got $STATUS"
  fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
section "Manual Test Instructions"
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo ""
echo -e "${YELLOW}⚠️  Manual browser tests required:${NC}"
echo ""
echo "1. Open browser (incognito mode recommended)"
echo "   ${BLUE}http://localhost:3000${NC}"
echo ""
echo "2. Test Case: TC-WEB-001 - Login & Session Persistence"
echo "   a) Should redirect to /login"
echo "   b) Enter: admin@rdcs-test.local / test123"
echo "   c) Click Login → should go to /dashboard"
echo "   d) Open DevTools (F12) → Application → Local Storage"
echo "      ✓ Check: rdcs_access_token exists"
echo "      ✓ Check: rdcs_refresh_token exists"
echo "   e) Refresh page (F5)"
echo "      ✓ Check: Stays on /dashboard (NO redirect to login)"
echo "   f) Open Console → Check logs:"
echo "      ✓ [authStore] Session restored, isAuthenticated = true"
echo "      ✓ [ProtectedRoute] Check auth: true"
echo ""
echo "3. Test Case: TC-WEB-002 - Logout"
echo "   a) Click Logout button"
echo "   b) Should redirect to /login"
echo "   c) Check Local Storage:"
echo "      ✓ rdcs_access_token removed"
echo "      ✓ rdcs_refresh_token removed"
echo "   d) Try to visit /dashboard"
echo "      ✓ Should redirect to /login"
echo ""
echo "4. Test Case: TC-WEB-003 - Token Expiration"
echo "   a) Login again"
echo "   b) Open DevTools → Application → Local Storage"
echo "   c) Manually delete rdcs_access_token"
echo "   d) Navigate to /devices"
echo "      ✓ Should get 401 and redirect to /login"
echo ""
echo "5. Test Case: TC-WEB-004 - Cross-Tab Session"
echo "   a) Login in Tab 1"
echo "   b) Open new tab → http://localhost:3000/dashboard"
echo "      ✓ Should show dashboard without login"
echo ""
echo "6. Test Case: TC-WEB-005 - Invalid Credentials"
echo "   a) Go to /login"
echo "   b) Enter: admin@rdcs-test.local / wrongpassword"
echo "      ✓ Should show error message"
echo "      ✓ Should stay on /login"
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Summary
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

TOTAL=$((PASS + FAIL))
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  Automated Test Results"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  Total: $TOTAL  ${GREEN}✅ Passed: $PASS${NC}  ${RED}❌ Failed: $FAIL${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
  echo -e "${GREEN}✅ All automated tests passed!${NC}"
  echo ""
  echo "Next: Execute manual tests in browser (see instructions above)"
else
  echo -e "${RED}❌ Some automated tests failed${NC}"
  echo "Fix issues before proceeding to manual tests"
  exit 1
fi

echo ""
echo "Test Accounts:"
echo "  Admin:   admin@rdcs-test.local / test123"
echo "  Manager: manager@rdcs-test.local / test123"
echo "  Member:  member@rdcs-test.local / test123"
echo ""
