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
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v5"

	"github.com/rdcs/rdcs-api/internal/model"
)

const (
	// AccessTokenExpiry is the lifetime of an access token.
	AccessTokenExpiry = 15 * time.Minute
	// RefreshTokenExpiry is the lifetime of a refresh token.
	RefreshTokenExpiry = 7 * 24 * time.Hour
	// rsaKeyBits is the RSA key size used by GenerateRSAKeyPair.
	rsaKeyBits = 2048
)

// Claims extends jwt.RegisteredClaims with RDCS-specific fields.
type Claims struct {
	jwt.RegisteredClaims
	MemberID string `json:"member_id"`
	TeamID   string `json:"team_id"`
	Role     string `json:"role"`
}

// TokenPair holds an access token and a refresh token together with
// the access token's remaining lifetime in seconds.
type TokenPair struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	ExpiresIn    int    `json:"expires_in"` // seconds
}

// GenerateTokenPair creates a signed access token and refresh token for
// the given member using RS256.  The access token contains member_id,
// team_id, and role; the refresh token contains only member_id.
func GenerateTokenPair(member *model.Member, privateKeyPEM string) (*TokenPair, error) {
	privKey, err := parsePrivateKey(privateKeyPEM)
	if err != nil {
		return nil, fmt.Errorf("parse private key: %w", err)
	}

	now := time.Now()

	// Access token: 15-minute lifetime with full claims.
	accessClaims := Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   member.ID.String(),
			IssuedAt:  jwt.NewNumericDate(now),
			ExpiresAt: jwt.NewNumericDate(now.Add(AccessTokenExpiry)),
			Issuer:    "rdcs-api",
		},
		MemberID: member.ID.String(),
		TeamID:   member.TeamID.String(),
		Role:     member.Role,
	}

	accessToken := jwt.NewWithClaims(jwt.SigningMethodRS256, accessClaims)
	accessTokenStr, err := accessToken.SignedString(privKey)
	if err != nil {
		return nil, fmt.Errorf("sign access token: %w", err)
	}

	// Refresh token: 7-day lifetime with member_id only.
	refreshClaims := jwt.RegisteredClaims{
		Subject:   member.ID.String(),
		IssuedAt:  jwt.NewNumericDate(now),
		ExpiresAt: jwt.NewNumericDate(now.Add(RefreshTokenExpiry)),
		Issuer:    "rdcs-api",
	}

	refreshToken := jwt.NewWithClaims(jwt.SigningMethodRS256, refreshClaims)
	refreshTokenStr, err := refreshToken.SignedString(privKey)
	if err != nil {
		return nil, fmt.Errorf("sign refresh token: %w", err)
	}

	return &TokenPair{
		AccessToken:  accessTokenStr,
		RefreshToken: refreshTokenStr,
		ExpiresIn:    int(AccessTokenExpiry.Seconds()),
	}, nil
}

// ValidateToken parses and validates an RS256-signed JWT string using
// the provided PEM-encoded public key.  It returns the embedded Claims
// on success or an error describing the validation failure.
func ValidateToken(tokenString string, publicKeyPEM string) (*Claims, error) {
	pubKey, err := parsePublicKey(publicKeyPEM)
	if err != nil {
		return nil, fmt.Errorf("parse public key: %w", err)
	}

	token, err := jwt.ParseWithClaims(tokenString, &Claims{}, func(t *jwt.Token) (interface{}, error) {
		// Ensure the signing method is RS256.
		if _, ok := t.Method.(*jwt.SigningMethodRSA); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		return pubKey, nil
	})
	if err != nil {
		return nil, fmt.Errorf("parse token: %w", err)
	}

	claims, ok := token.Claims.(*Claims)
	if !ok || !token.Valid {
		return nil, fmt.Errorf("invalid token claims")
	}

	return claims, nil
}

// GenerateRSAKeyPair creates a new 2048-bit RSA key pair and returns
// both keys as PEM-encoded strings.  This is intended for development
// setup and initial provisioning; production deployments should manage
// keys through their own secrets infrastructure.
func GenerateRSAKeyPair() (privateKey string, publicKey string, err error) {
	rsaKey, err := rsa.GenerateKey(rand.Reader, rsaKeyBits)
	if err != nil {
		return "", "", fmt.Errorf("generate rsa key: %w", err)
	}

	privBytes := x509.MarshalPKCS1PrivateKey(rsaKey)
	privPEM := pem.EncodeToMemory(&pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: privBytes,
	})

	pubBytes, err := x509.MarshalPKIXPublicKey(&rsaKey.PublicKey)
	if err != nil {
		return "", "", fmt.Errorf("marshal public key: %w", err)
	}
	pubPEM := pem.EncodeToMemory(&pem.Block{
		Type:  "RSA PUBLIC KEY",
		Bytes: pubBytes,
	})

	return string(privPEM), string(pubPEM), nil
}

// parsePrivateKey decodes a PEM-encoded RSA private key.
func parsePrivateKey(pemStr string) (*rsa.PrivateKey, error) {
	// Debug: log first 100 chars to check for literal \n
	if len(pemStr) > 100 {
		fmt.Printf("DEBUG parsePrivateKey: first 100 chars: %q\n", pemStr[:100])
	} else {
		fmt.Printf("DEBUG parsePrivateKey: full key: %q\n", pemStr)
	}

	block, _ := pem.Decode([]byte(pemStr))
	if block == nil {
		return nil, fmt.Errorf("failed to decode PEM block")
	}
	key, err := x509.ParsePKCS1PrivateKey(block.Bytes)
	if err != nil {
		return nil, fmt.Errorf("parse pkcs1 private key: %w", err)
	}
	return key, nil
}

// parsePublicKey decodes a PEM-encoded RSA public key.
func parsePublicKey(pemStr string) (*rsa.PublicKey, error) {
	block, _ := pem.Decode([]byte(pemStr))
	if block == nil {
		return nil, fmt.Errorf("failed to decode PEM block")
	}
	pub, err := x509.ParsePKIXPublicKey(block.Bytes)
	if err != nil {
		return nil, fmt.Errorf("parse pkix public key: %w", err)
	}
	rsaPub, ok := pub.(*rsa.PublicKey)
	if !ok {
		return nil, fmt.Errorf("key is not RSA")
	}
	return rsaPub, nil
}
