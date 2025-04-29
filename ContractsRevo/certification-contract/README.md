# Certification Management Contract

## Overview
The Certification Management contract provides a secure, tamper-proof system for issuing, validating, and auditing certifications on the Stellar blockchain. It enables organizations to issue verifiable certifications with document hash verification, expiration dates, and comprehensive audit trails.

## Features
- **Secure Issuance**: Issue tamper-proof certifications with metadata and validity periods
- **Document Verification**: Verify certification authenticity using document hash validation
- **Status Management**: Track certification status (Valid, Expired, Revoked)
- **Comprehensive Auditing**: Generate detailed audit reports with various filtering options
- **Issuer Verification**: Only authorized issuers can issue certifications
- **Expiration Handling**: Automatic expiration date validation and status updates

## Contract Components
The contract consists of several modules:

- **lib.rs**: Main contract implementation with public functions
- **datatypes.rs**: Data structures, errors, and key definitions
- **issuance.rs**: Handles certification issuance logic
- **validation.rs**: Manages validation and revocation of certifications
- **audit.rs**: Provides reporting and auditing capabilities
- **test.rs**: Comprehensive test suite

## Supported Certification Types
- Organic
- FairTrade
- UTZ
- RainforestAlliance
- ISO9001
- ISO14001
- HACCP
- Kosher
- Halal
- Demeter
- Custom (with Symbol)

## How It Works

### Certification Issuance
The issuer creates a certification by providing:
- Document hash (a cryptographic hash of the certification document)
- Metadata (key-value pairs of certification details)
- Validity period (start and end dates)
- Holder address (the entity receiving the certification)

### Certification Verification
Anyone can verify a certification by providing:
- Certification ID
- Document hash

The contract verifies:
1. The certification exists and is issued by a verified issuer
2. The document hash matches the stored hash
3. The certification is not expired or revoked

### Certification Revocation
Only the original issuer can revoke a certification by providing:
- Certification ID
- Reason for revocation

### Audit Reports
The contract provides functions to generate audit reports with optional filters:
- By certification type
- By issuer
- By status (Valid, Expired, Revoked)
- By holder

## Testing
The contract includes comprehensive tests covering:
- Certification issuance with valid and invalid parameters
- Document hash verification
- Expiration date validation
- Certification revocation and status transitions
- Audit report generation
- Edge cases and error handling

## Usage

### Initialize the Contract
```rust
client.initialize(&admin);
```

### Issue a Certification
```rust
let cert_id = client.issue_certification(
    &issuer,
    &holder,
    &CertificationType::Organic,
    &document_hash,
    &metadata,
    valid_from,
    valid_to,
);
```

### Verify a Certification
```rust
let is_valid = client.verify_certification(&cert_id, &document_hash);
```

### Revoke a Certification
```rust
client.revoke_certification(
    &issuer,
    &cert_id,
    &symbol_short!("violation")
);
```

### Generate Audit Report
```rust
let audit_report = client.generate_audit_report(
    &Some(CertificationType::Organic),
    &None,
    &None
);
``` 