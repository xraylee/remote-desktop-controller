// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package auth

import (
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"strings"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/model"
)

func TestGenerateRSAKeyPair(t *testing.T) {
	privPEM, pubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	// Verify private key PEM is valid.
	if !strings.Contains(privPEM, "RSA PRIVATE KEY") {
		t.Error("private key PEM does not contain expected header")
	}
	privBlock, _ := pem.Decode([]byte(privPEM))
	if privBlock == nil {
		t.Fatal("failed to decode private key PEM")
	}
	privKey, err := x509.ParsePKCS1PrivateKey(privBlock.Bytes)
	if err != nil {
		t.Fatalf("failed to parse private key: %v", err)
	}
	if privKey.N.BitLen() != rsaKeyBits {
		t.Errorf("expected %d-bit key, got %d-bit", rsaKeyBits, privKey.N.BitLen())
	}

	// Verify public key PEM is valid.
	if !strings.Contains(pubPEM, "RSA PUBLIC KEY") {
		t.Error("public key PEM does not contain expected header")
	}
	pubBlock, _ := pem.Decode([]byte(pubPEM))
	if pubBlock == nil {
		t.Fatal("failed to decode public key PEM")
	}
	pub, err := x509.ParsePKIXPublicKey(pubBlock.Bytes)
	if err != nil {
		t.Fatalf("failed to parse public key: %v", err)
	}
	if _, ok := pub.(*rsa.PublicKey); !ok {
		t.Error("public key is not RSA")
	}
}

func newTestMember() *model.Member {
	return &model.Member{
		ID:     uuid.MustParse("11111111-1111-1111-1111-111111111111"),
		TeamID: uuid.MustParse("22222222-2222-2222-2222-222222222222"),
		Name:   "Test User",
		Email:  "test@example.com",
		Role:   "admin",
	}
}

func TestGenerateTokenPair(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMember()
	tp, err := GenerateTokenPair(member, privPEM)
	if err != nil {
		t.Fatalf("GenerateTokenPair() error = %v", err)
	}

	if tp.AccessToken == "" {
		t.Error("access token is empty")
	}
	if tp.RefreshToken == "" {
		t.Error("refresh token is empty")
	}
	if tp.ExpiresIn != int(AccessTokenExpiry.Seconds()) {
		t.Errorf("ExpiresIn = %d, want %d", tp.ExpiresIn, int(AccessTokenExpiry.Seconds()))
	}
}

func TestValidateToken(t *testing.T) {
	privPEM, pubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMember()
	tp, err := GenerateTokenPair(member, privPEM)
	if err != nil {
		t.Fatalf("GenerateTokenPair() error = %v", err)
	}

	claims, err := ValidateToken(tp.AccessToken, pubPEM)
	if err != nil {
		t.Fatalf("ValidateToken() error = %v", err)
	}

	if claims.MemberID != member.ID.String() {
		t.Errorf("MemberID = %q, want %q", claims.MemberID, member.ID.String())
	}
	if claims.TeamID != member.TeamID.String() {
		t.Errorf("TeamID = %q, want %q", claims.TeamID, member.TeamID.String())
	}
	if claims.Role != member.Role {
		t.Errorf("Role = %q, want %q", claims.Role, member.Role)
	}
	if claims.Issuer != "rdcs-api" {
		t.Errorf("Issuer = %q, want %q", claims.Issuer, "rdcs-api")
	}
}

func TestValidateToken_Expired(t *testing.T) {
	privPEM, pubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMember()

	// Manually create an expired token.
	expiredClaims := Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   member.ID.String(),
			IssuedAt:  jwt.NewNumericDate(time.Now().Add(-2 * time.Hour)),
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(-1 * time.Hour)),
			Issuer:    "rdcs-api",
		},
		MemberID: member.ID.String(),
		TeamID:   member.TeamID.String(),
		Role:     member.Role,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodRS256, expiredClaims)
	privKey, _ := parsePrivateKey(privPEM)
	tokenStr, err := token.SignedString(privKey)
	if err != nil {
		t.Fatalf("failed to sign expired token: %v", err)
	}

	_, err = ValidateToken(tokenStr, pubPEM)
	if err == nil {
		t.Fatal("ValidateToken() expected error for expired token, got nil")
	}
	if !strings.Contains(err.Error(), "parse token") {
		t.Errorf("expected parse token error, got: %v", err)
	}
}

func TestValidateToken_WrongKey(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	// Generate a different key pair for validation.
	_, otherPubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMember()
	tp, err := GenerateTokenPair(member, privPEM)
	if err != nil {
		t.Fatalf("GenerateTokenPair() error = %v", err)
	}

	_, err = ValidateToken(tp.AccessToken, otherPubPEM)
	if err == nil {
		t.Fatal("ValidateToken() expected error for wrong key, got nil")
	}
}

func TestValidateToken_InvalidPEM(t *testing.T) {
	_, err := ValidateToken("some.token.string", "not-a-pem-key")
	if err == nil {
		t.Fatal("ValidateToken() expected error for invalid PEM, got nil")
	}
}

func TestValidateToken_MalformedToken(t *testing.T) {
	_, pubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	_, err = ValidateToken("not.a.valid.jwt", pubPEM)
	if err == nil {
		t.Fatal("ValidateToken() expected error for malformed token, got nil")
	}
}

func TestGenerateTokenPair_InvalidPrivateKey(t *testing.T) {
	member := newTestMember()
	_, err := GenerateTokenPair(member, "not-a-valid-key")
	if err == nil {
		t.Fatal("GenerateTokenPair() expected error for invalid private key, got nil")
	}
}

func TestValidateRefreshToken(t *testing.T) {
	privPEM, pubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMember()
	tp, err := GenerateTokenPair(member, privPEM)
	if err != nil {
		t.Fatalf("GenerateTokenPair() error = %v", err)
	}

	// Parse refresh token with RegisteredClaims (not full Claims).
	pubKey, _ := parsePublicKey(pubPEM)
	token, err := jwt.ParseWithClaims(tp.RefreshToken, &jwt.RegisteredClaims{}, func(tok *jwt.Token) (interface{}, error) {
		return pubKey, nil
	})
	if err != nil {
		t.Fatalf("failed to parse refresh token: %v", err)
	}

	claims, ok := token.Claims.(*jwt.RegisteredClaims)
	if !ok || !token.Valid {
		t.Fatal("refresh token claims are invalid")
	}

	if claims.Subject != member.ID.String() {
		t.Errorf("refresh token Subject = %q, want %q", claims.Subject, member.ID.String())
	}
}
