# Certificate Management Contract

## 🎯 Overview
The Certificate Management Contract is a decentralized smart contract built on the Soroban framework for the Stellar blockchain. It enables the issuance, verification, and management of digital certificates for agricultural products and practices. The contract ensures the authenticity, validity, and traceability of certifications by leveraging blockchain technology.

## 📜 Features
- Issue digital certifications with cryptographic verification
- Verify certificate authenticity using document hashes
- Track certificate status (Valid, Expired, Revoked)
- Manage certificate lifecycle including expiration
- Generate audit reports for compliance and transparency
- Secure certificate revocation by authorized issuers

## 🛠 Contract Functionality
### **1. Certificate Issuance**
The contract allows authorized issuers to:
- Create digital certificates for recipients
- Specify certificate types (Organic, Fair Trade, etc.)
- Set expiration dates for certificates
- Store cryptographic hashes of certification documents
- Assign unique IDs to each certificate

### **2. Certificate Verification**
Users can verify certificates by:
- Submitting document hashes for verification
- Checking certificate status (Valid, Expired, Revoked)
- Confirming certificate authenticity
- Validating certificate expiration dates

### **3. Certificate Management**
The contract provides functionality to:
- Revoke certificates by authorized issuers
- Expire certificates automatically based on expiration dates
- Update certificate status as needed
- Track certificate history and changes

### **4. Audit and Reporting**
The contract includes audit capabilities:
- Generate comprehensive certificate audit reports
- Filter reports by certificate status
- Filter reports by issuer
- Filter reports by timestamp
- Track certificate lifecycle events

## 🚀 Setup Guide
### **Prerequisites**
Ensure you have the following installed:
- Rust & Cargo
- Soroban CLI
- Stellar Standalone/Testnet/Mainnet access
- Node.js (for interacting with the contract via scripts)

### **Installation Steps**
1. **Clone the Repository**
   ```bash
   git clone https://github.com/Crypto-Jaguars/Revo-Contracts.git
   cd ContractsRevo/certificate-management-contract
   ```
2. **Build the Contract**
   ```bash
   stellar contract build
   ```
3. **Run the Tests**
   ```bash
   cargo test
   ```

NB: Contract deployed to Testnet on `CATFMGOLSDM4ZZX4L6POV2JPJOLHSWKHYW3ZSBEQFSXPZYVBP5UZZSQU`

## 📊 Data Structures
### **Certification**
Represents a digital certificate with the following properties:
- ID: Unique identifier for the certificate
- Certificate Type: Symbol representing the certification type (e.g., "Organic", "Fair Trade")
- Issuer: Address of the certifying authority
- Issued Date: Timestamp when the certificate was issued
- Expiration Date: Timestamp when the certificate expires
- Verification Hash: Cryptographic hash of certification documents
- Status: Current status of the certificate (Valid, Expired, Revoked)

### **CertStatus**
Enum representing the possible states of a certificate:
- Valid: Certificate is currently valid and active
- Expired: Certificate has reached its expiration date
- Revoked: Certificate has been manually revoked by the issuer

## 📌 Best Practices
- Ensure proper authorization before issuing or revoking certificates
- Use secure methods to generate document hashes
- Set appropriate expiration dates based on certification standards
- Regularly audit certificate status for compliance
- Verify certificate authenticity before accepting certified products

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Admin operations (unauthorized access, uninitialized contract)
- Certificate issuance (invalid parameters, unauthorized issuers)
- Certificate revocation (already revoked, unauthorized revocation)
- Certificate verification (hash mismatch, expired certificates)
- Certificate expiration (already expired, not yet expired)
- Audit operations (invalid parameters, unauthorized access)

## 🔄 Contract Interactions
### **For Issuers**
1. Issue new certificates to recipients
2. Revoke certificates when necessary
3. Generate audit reports for issued certificates

### **For Certificate Holders**
1. Verify certificate authenticity
2. Check certificate status
3. Generate audit reports for owned certificates

### **For Verifiers**
1. Verify certificate authenticity using document hashes
2. Check certificate status and expiration
3. Validate issuer authority

## 🌐 Use Cases
- Organic certification for agricultural products
- Fair Trade certification for ethical sourcing
- Quality assurance certifications for food products
- Sustainability certifications for farming practices
- Origin verification for regional products
- Compliance certifications for regulatory requirements

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
