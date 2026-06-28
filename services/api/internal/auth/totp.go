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
	"encoding/base32"
	"fmt"
	"net/url"
	"time"

	"github.com/pquerna/otp"
	"github.com/pquerna/otp/totp"
)

const (
	// TOTP_DIGITS is the number of digits in a TOTP code.
	TOTP_DIGITS = 6
	// TOTP_PERIOD is the time step in seconds for TOTP code rotation.
	TOTP_PERIOD = 30
	// TOTP_SECRET_LENGTH is the number of random bytes used to generate a TOTP secret.
	TOTP_SECRET_LENGTH = 20
)

// GenerateTOTPSecret generates a cryptographically random 20-byte secret
// and returns it as a base32-encoded string (no padding) suitable for use
// with TOTP authenticator applications.
func GenerateTOTPSecret() (string, error) {
	secret := make([]byte, TOTP_SECRET_LENGTH)
	if _, err := rand.Read(secret); err != nil {
		return "", fmt.Errorf("generate random bytes: %w", err)
	}
	return base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(secret), nil
}

// GenerateTOTPURI builds an otpauth:// URI that can be encoded into a QR code
// for enrollment with standard TOTP authenticator applications (Google
// Authenticator, Authy, etc.).
func GenerateTOTPURI(secret string, email string, issuer string) string {
	// Decode the base32 secret to raw bytes so the library can re-encode
	// it correctly when building the URI.
	rawSecret, err := base32.StdEncoding.WithPadding(base32.NoPadding).DecodeString(secret)
	if err != nil {
		// Fall back to manual construction on decode failure.
		return fmt.Sprintf(
			"otpauth://totp/%s:%s?secret=%s&issuer=%s&algorithm=SHA1&digits=%d&period=%d",
			url.PathEscape(issuer),
			url.PathEscape(email),
			secret,
			url.QueryEscape(issuer),
			TOTP_DIGITS,
			TOTP_PERIOD,
		)
	}

	// Use the otp.Key builder for correct URI encoding.
	opts := totp.GenerateOpts{
		Issuer:      issuer,
		AccountName: email,
		Period:      TOTP_PERIOD,
		Digits:      otp.DigitsSix,
		Algorithm:   otp.AlgorithmSHA1,
		Secret:      rawSecret,
	}
	key, err := totp.Generate(opts)
	if err != nil {
		// Fall back to manual construction if the library call fails.
		return fmt.Sprintf(
			"otpauth://totp/%s:%s?secret=%s&issuer=%s&algorithm=SHA1&digits=%d&period=%d",
			url.PathEscape(issuer),
			url.PathEscape(email),
			secret,
			url.QueryEscape(issuer),
			TOTP_DIGITS,
			TOTP_PERIOD,
		)
	}
	return key.URL()
}

// ValidateTOTPCode validates a 6-digit TOTP code against the given base32-encoded
// secret using HMAC-SHA1 with a 30-second time step.  Codes from the current and
// adjacent time windows are accepted (±30 second tolerance, i.e. skew=1).
func ValidateTOTPCode(secret string, code string) bool {
	return totp.Validate(code, secret)
}

// ValidateTOTPCodeAtTime validates a TOTP code against a specific time rather
// than time.Now().  This is primarily useful for testing.
func ValidateTOTPCodeAtTime(secret string, code string, t time.Time) bool {
	rv, _ := totp.ValidateCustom(code, secret, t, totp.ValidateOpts{
		Period:    TOTP_PERIOD,
		Skew:      1,
		Digits:    otp.DigitsSix,
		Algorithm: otp.AlgorithmSHA1,
	})
	return rv
}

// GenerateTOTPCodeAtTime generates a valid TOTP code for the given secret at
// the specified time.  This is primarily useful for testing.
func GenerateTOTPCodeAtTime(secret string, t time.Time) (string, error) {
	return totp.GenerateCode(secret, t)
}
