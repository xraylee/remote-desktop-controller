# Web Admin Frontend - Test Plan

**Project:** RDCS Web Admin  
**Date:** 2026-06-29  
**Scope:** Authentication system and login persistence

---

## Test Environment

- **Frontend:** React + TypeScript + Vite + Zustand
- **API:** Go REST API on port 8080
- **Database:** PostgreSQL (Docker)
- **Test Accounts:**
  - Admin: `admin@rdcs-test.local` / `test123`
  - Manager: `manager@rdcs-test.local` / `test123`
  - Member: `member@rdcs-test.local` / `test123`

---

## 1. Unit Tests

### 1.1 AuthStore Tests

**Location:** `web/admin/src/stores/__tests__/authStore.test.ts`

#### Test Cases

| ID | Test Case | Input | Expected Output | Status |
|----|-----------|-------|----------------|--------|
| UT-AUTH-001 | Initialize with no tokens | Empty localStorage | `isAuthenticated = false` | ⏳ |
| UT-AUTH-002 | Initialize with stored tokens | Valid tokens in localStorage | `isAuthenticated = true` | ⏳ |
| UT-AUTH-003 | Login success | Valid credentials | Token saved, `isAuthenticated = true` | ⏳ |
| UT-AUTH-004 | Login failure | Invalid credentials | No token, `isAuthenticated = false` | ⏳ |
| UT-AUTH-005 | Logout | Authenticated state | Tokens cleared, `isAuthenticated = false` | ⏳ |
| UT-AUTH-006 | restoreSession with tokens | Tokens in localStorage | Session restored | ⏳ |
| UT-AUTH-007 | restoreSession without tokens | Empty localStorage | No session restored | ⏳ |

### 1.2 API Client Tests

**Location:** `web/admin/src/api/__tests__/client.test.ts`

#### Test Cases

| ID | Test Case | Expected Behavior | Status |
|----|-----------|-------------------|--------|
| UT-API-001 | Request with access token | Authorization header present | ⏳ |
| UT-API-002 | Request without access token | No Authorization header | ⏳ |
| UT-API-003 | 401 response handling | Redirect to /login | ⏳ |
| UT-API-004 | setAccessToken() | Token stored in memory | ⏳ |

### 1.3 ProtectedRoute Tests

**Location:** `web/admin/src/components/__tests__/ProtectedRoute.test.tsx`

#### Test Cases

| ID | Test Case | Auth State | Expected Behavior | Status |
|----|-----------|-----------|-------------------|--------|
| UT-ROUTE-001 | Render with auth | `isAuthenticated = true` | Render children | ⏳ |
| UT-ROUTE-002 | Render without auth | `isAuthenticated = false` | Redirect to /login | ⏳ |
| UT-ROUTE-003 | Preserve return URL | Unauthenticated user visits /dashboard | Redirect with state | ⏳ |

---

## 2. Integration Tests

### 2.1 Login Flow

**Location:** `web/admin/tests/integration/login.spec.ts`

#### Test Cases

| ID | Test Case | Steps | Expected Result | Status |
|----|-----------|-------|----------------|--------|
| IT-LOGIN-001 | Successful login | 1. Visit /login<br>2. Enter valid credentials<br>3. Submit | Redirect to /dashboard, tokens in localStorage | ⏳ |
| IT-LOGIN-002 | Failed login - wrong password | 1. Visit /login<br>2. Enter wrong password<br>3. Submit | Error message, stay on /login | ⏳ |
| IT-LOGIN-003 | Failed login - user not found | 1. Visit /login<br>2. Enter non-existent email<br>3. Submit | Error message, stay on /login | ⏳ |
| IT-LOGIN-004 | Auto-redirect after login | 1. Visit /dashboard (not logged in)<br>2. Login | Redirect back to /dashboard | ⏳ |

### 2.2 Session Persistence

**Critical Test Suite**

| ID | Test Case | Steps | Expected Result | Status |
|----|-----------|-------|----------------|--------|
| IT-PERSIST-001 | Page refresh keeps session | 1. Login<br>2. Go to /dashboard<br>3. Refresh (F5) | Stay on /dashboard, still authenticated | ⏳ |
| IT-PERSIST-002 | New tab restores session | 1. Login in tab A<br>2. Open new tab B<br>3. Visit /dashboard | Access granted in tab B | ⏳ |
| IT-PERSIST-003 | Close and reopen browser | 1. Login<br>2. Close browser<br>3. Reopen, visit /dashboard | Session restored | ⏳ |
| IT-PERSIST-004 | Tokens in localStorage | After login | `rdcs_access_token` and `rdcs_refresh_token` present | ⏳ |

### 2.3 Logout Flow

| ID | Test Case | Steps | Expected Result | Status |
|----|-----------|-------|----------------|--------|
| IT-LOGOUT-001 | Manual logout | 1. Login<br>2. Click logout | Redirect to /login, tokens cleared | ⏳ |
| IT-LOGOUT-002 | Logout clears localStorage | After logout | `rdcs_access_token` and `rdcs_refresh_token` removed | ⏳ |
| IT-LOGOUT-003 | Cannot access protected routes | 1. Logout<br>2. Visit /dashboard | Redirect to /login | ⏳ |

### 2.4 Token Expiration

| ID | Test Case | Steps | Expected Result | Status |
|----|-----------|-------|----------------|--------|
| IT-TOKEN-001 | Expired access token | 1. Login<br>2. Wait 16 min (token expires at 15 min)<br>3. API request | 401 → redirect to /login | ⏳ |
| IT-TOKEN-002 | Manual token removal | 1. Login<br>2. Delete localStorage tokens<br>3. Refresh | Redirect to /login | ⏳ |

### 2.5 Authorization

| ID | Test Case | Steps | Expected Result | Status |
|----|-----------|-------|----------------|--------|
| IT-AUTH-001 | Owner access all pages | Login as owner | All menu items accessible | ⏳ |
| IT-AUTH-002 | Member limited access | Login as member | Cannot invite members (403) | ⏳ |

---

## 3. Manual Test Checklist

### 3.1 Pre-Test Setup

```bash
# 1. Start API
cd ~/Development/source/remote-desktop-controller/services/api
go run cmd/api/main.go

# 2. Start Web
cd ~/Development/source/remote-desktop-controller/web/admin
npm run dev

# 3. Verify API health
curl http://localhost:8080/healthz
```

### 3.2 Test Execution

#### ✅ TC-WEB-001: Login and Session Persistence

**Steps:**
1. Open browser in **incognito mode** → `http://localhost:3000`
2. Should redirect to `/login`
3. Enter credentials: `admin@rdcs-test.local` / `test123`
4. Click "Login"
5. Verify redirect to `/dashboard`
6. **Open DevTools (F12) → Application → Local Storage**
   - ✓ `rdcs_access_token` exists
   - ✓ `rdcs_refresh_token` exists
7. **Refresh page (F5)**
   - ✓ Stays on `/dashboard`
   - ✓ No redirect to login
8. **Open Console, check logs:**
   - ✓ `[authStore] Session restored, isAuthenticated = true`
   - ✓ `[ProtectedRoute] Check auth: true`

**Expected:** ✅ All checks pass  
**Actual:** ⏳ _To be tested_

---

#### ✅ TC-WEB-002: Logout Clears Session

**Steps:**
1. While logged in, click "Logout" button
2. Verify redirect to `/login`
3. **Check DevTools → Local Storage**
   - ✓ `rdcs_access_token` removed
   - ✓ `rdcs_refresh_token` removed
4. Try to visit `/dashboard` directly
   - ✓ Redirects to `/login`

**Expected:** ✅ All checks pass  
**Actual:** ⏳ _To be tested_

---

#### ✅ TC-WEB-003: Expired Token Handling

**Steps:**
1. Login successfully
2. **Manually delete `rdcs_access_token` from localStorage**
3. Navigate to any page (e.g., `/devices`)
4. Verify:
   - ✓ API returns 401
   - ✓ Redirect to `/login`

**Expected:** ✅ Auto logout on 401  
**Actual:** ⏳ _To be tested_

---

#### ✅ TC-WEB-004: Cross-Tab Session Sync

**Steps:**
1. Login in Tab A
2. Open new tab (Tab B) → `http://localhost:3000/dashboard`
3. Verify:
   - ✓ Tab B shows dashboard without login prompt

**Expected:** ✅ Session shared across tabs  
**Actual:** ⏳ _To be tested_

---

#### ✅ TC-WEB-005: Invalid Credentials

**Steps:**
1. Go to `/login`
2. Enter: `admin@rdcs-test.local` / `wrongpassword`
3. Click "Login"
4. Verify:
   - ✓ Error message displayed
   - ✓ Still on `/login` page
   - ✓ No tokens in localStorage

**Expected:** ✅ Login fails gracefully  
**Actual:** ⏳ _To be tested_

---

## 4. Automated Test Script

**Location:** `web/admin/tests/manual/test-auth-flow.sh`

```bash
#!/bin/bash
# Automated auth flow test

API_URL="http://localhost:8080"
WEB_URL="http://localhost:3000"

echo "🧪 RDCS Web Auth Flow Test"
echo "=============================="

# TC-1: API Health
echo -n "1. API Health Check... "
if curl -s "$API_URL/healthz" | grep -q "ok"; then
  echo "✅"
else
  echo "❌ API not running"
  exit 1
fi

# TC-2: Web is accessible
echo -n "2. Web UI accessible... "
if curl -s -o /dev/null -w "%{http_code}" "$WEB_URL" | grep -q "200"; then
  echo "✅"
else
  echo "❌ Web UI not running"
  exit 1
fi

# TC-3: Login API works
echo -n "3. Login API... "
RESP=$(curl -s -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@rdcs-test.local","password":"test123"}')

if echo "$RESP" | grep -q "access_token"; then
  echo "✅"
else
  echo "❌ Login failed"
  echo "Response: $RESP"
  exit 1
fi

# TC-4: Invalid credentials
echo -n "4. Invalid credentials rejected... "
RESP=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@rdcs-test.local","password":"wrong"}')
STATUS=$(echo "$RESP" | tail -n 1)

if [ "$STATUS" = "401" ]; then
  echo "✅"
else
  echo "❌ Should return 401"
  exit 1
fi

echo ""
echo "✅ All automated tests passed!"
echo ""
echo "📋 Manual tests required:"
echo "  1. Open http://localhost:3000 in browser"
echo "  2. Login with admin@rdcs-test.local / test123"
echo "  3. Refresh page (F5) - should stay logged in"
echo "  4. Check DevTools → Application → Local Storage for tokens"
```

---

## 5. Test Results Summary

### Unit Tests
- Total: 15
- Passed: ⏳
- Failed: ⏳
- Coverage: ⏳

### Integration Tests  
- Total: 18
- Passed: ⏳
- Failed: ⏳

### Manual Tests
- Total: 5
- Passed: ⏳
- Failed: ⏳

---

## 6. Known Issues

| ID | Issue | Severity | Status |
|----|-------|----------|--------|
| - | - | - | - |

---

## 7. Next Steps

1. ✅ Create unit test files with Vitest
2. ✅ Implement integration tests with Playwright
3. ✅ Run automated test script
4. ✅ Execute manual test checklist
5. ✅ Document results
6. ✅ Fix any failures
7. ✅ Final validation

---

## Appendix: Test Data

### Test Accounts

```sql
-- Already in database
SELECT email, role FROM members;

-- admin@rdcs-test.local  | owner
-- manager@rdcs-test.local | manager  
-- member@rdcs-test.local  | member
```

### JWT Tokens (Example)

```
Access Token Lifetime: 15 minutes
Refresh Token Lifetime: 7 days
Algorithm: RS256
```
