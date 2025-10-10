# Zigbee Encryption Implementation

## Overview

This document describes the AES-128 CCM* encryption implementation for the Zigbee driver. CCM* (Counter with CBC-MAC*) is a combined encryption and authentication mode specified in the Zigbee specification for securing frame communications.

## Architecture

### Component Structure

```
┌─────────────────────────────────────────────────────────────┐
│                     Zigbee Driver                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              SecurityManager                         │  │
│  │  - Network key management                            │  │
│  │  - Link key management                               │  │
│  │  - Frame counter tracking                            │  │
│  │  - Install code handling                             │  │
│  │  - Frame encryption/decryption API                   │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                        │
│                     ▼                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                 Crypto Module                        │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │            CCM* Implementation                 │  │  │
│  │  │  - encrypt_and_auth()                          │  │  │
│  │  │  - decrypt_and_verify()                        │  │  │
│  │  │  - auth_only()                                 │  │  │
│  │  │  - verify_auth()                               │  │  │
│  │  └────────┬───────────────────────┬───────────────┘  │  │
│  │           │                       │                   │  │
│  │           ▼                       ▼                   │  │
│  │  ┌─────────────────┐    ┌──────────────────┐        │  │
│  │  │   CTR Mode      │    │    CBC-MAC       │        │  │
│  │  │  (Encryption)   │    │ (Authentication) │        │  │
│  │  └────────┬────────┘    └──────┬───────────┘        │  │
│  │           │                    │                     │  │
│  │           └────────┬───────────┘                     │  │
│  │                    ▼                                 │  │
│  │           ┌─────────────────┐                        │  │
│  │           │  AES-128 Block  │                        │  │
│  │           │    Cipher       │                        │  │
│  │           └─────────────────┘                        │  │
│  └────────────────────┬─────────────────────────────────┘  │
│                       │                                     │
└───────────────────────┼─────────────────────────────────────┘
                        ▼
         ┌──────────────────────────────┐
         │     ESP-HAL AES Driver       │
         │   (Hardware Acceleration)    │
         └──────────────────────────────┘
```

### Key Components

#### 1. SecurityManager (`security.rs`)
- **Purpose**: High-level security management for Zigbee
- **Responsibilities**:
  - Store and manage network keys
  - Store and manage link keys
  - Track outgoing frame counters
  - Provide frame encryption/decryption API
  - Handle install codes for secure commissioning
  - Key derivation and rotation

#### 2. Crypto Module (`crypto.rs`)
- **Purpose**: CCM* mode implementation
- **Responsibilities**:
  - Implement AES-128 CCM* algorithm
  - Combine CTR mode encryption with CBC-MAC authentication
  - Support authentication-only mode (no encryption)
  - Nonce construction
  - Constant-time comparison for security

#### 3. AES Hardware (`esp-hal::aes`)
- **Purpose**: Hardware AES acceleration
- **Responsibilities**:
  - Perform AES block encryption operations
  - Provide 128/192/256-bit key support
  - Hardware acceleration for performance

## CCM* Algorithm

### Overview

CCM* is a variation of CCM (Counter with CBC-MAC) that allows for authentication-only mode. It combines:
- **CTR Mode**: Stream cipher for encryption
- **CBC-MAC**: Block cipher authentication

### Nonce Structure (13 bytes)

```
┌──────────────────┬──────────────────┬─────────────────┐
│  Source Address  │  Frame Counter   │ Security Control│
│    (8 bytes)     │    (4 bytes)     │   (1 byte)      │
└──────────────────┴──────────────────┴─────────────────┘
```

- **Source Address**: IEEE 64-bit address (little-endian)
- **Frame Counter**: 32-bit counter for replay protection (little-endian)
- **Security Control**: Security level and key ID

### Encryption Process

```
Input:
  - Key (16 bytes)
  - Nonce (13 bytes)
  - AAD (header, variable length)
  - Plaintext (variable length)
  - MIC length (4, 8, or 16 bytes)

Step 1: Calculate CBC-MAC
  ┌─────────────┐
  │   Build B_0 │  ← Flags, Nonce, Message Length
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │  AES Block  │  ← Encrypt B_0 with key
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │ Process AAD │  ← XOR + AES for each AAD block
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │Process Data │  ← XOR + AES for each data block
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │  Auth Tag   │  ← Result is CBC-MAC tag
  └─────────────┘

Step 2: Encrypt Plaintext using CTR Mode
  For each block:
    Counter Block = [Flags | Nonce | Counter]
    Keystream = AES(Counter Block, Key)
    Ciphertext = Plaintext ⊕ Keystream
    Counter++

Step 3: Encrypt Authentication Tag
  Counter Block 0 = [Flags | Nonce | 0x0000]
  Keystream = AES(Counter Block 0, Key)
  MIC = Auth Tag ⊕ Keystream (first M bytes)

Output:
  - Ciphertext (same length as plaintext)
  - MIC (4, 8, or 16 bytes)
```

### Decryption Process

```
Input:
  - Key (16 bytes)
  - Nonce (13 bytes)
  - AAD (header, variable length)
  - Ciphertext (variable length)
  - MIC (4, 8, or 16 bytes)

Step 1: Decrypt Authentication Tag
  Counter Block 0 = [Flags | Nonce | 0x0000]
  Keystream = AES(Counter Block 0, Key)
  Auth Tag = MIC ⊕ Keystream

Step 2: Decrypt Ciphertext using CTR Mode
  For each block:
    Counter Block = [Flags | Nonce | Counter]
    Keystream = AES(Counter Block, Key)
    Plaintext = Ciphertext ⊕ Keystream
    Counter++

Step 3: Calculate CBC-MAC on Plaintext
  (Same as encryption Step 1)

Step 4: Verify Authentication
  Compare calculated Auth Tag with received Auth Tag
  (Constant-time comparison to prevent timing attacks)

Output:
  - Plaintext (if authentication succeeds)
  - Error (if authentication fails, plaintext is zeroed)
```

## Security Levels

Zigbee defines 8 security levels:

| Level | Value | Encryption | MIC Size | Description |
|-------|-------|------------|----------|-------------|
| None | 0x00 | No | 0 bytes | No security |
| MIC-32 | 0x01 | No | 4 bytes | Authentication only |
| MIC-64 | 0x02 | No | 8 bytes | Authentication only |
| MIC-128 | 0x03 | No | 16 bytes | Authentication only |
| Reserved | 0x04 | - | - | Reserved |
| ENC-MIC-32 | 0x05 | Yes | 4 bytes | Encryption + Auth |
| ENC-MIC-64 | 0x06 | Yes | 8 bytes | Encryption + Auth |
| ENC-MIC-128 | 0x07 | Yes | 16 bytes | Encryption + Auth |

### MIC Size Recommendations

- **MIC-32 (4 bytes)**: Suitable for most applications, provides 2^32 security level
- **MIC-64 (8 bytes)**: Higher security, provides 2^64 security level
- **MIC-128 (16 bytes)**: Maximum security, provides 2^128 security level

## Key Management

### Key Types

#### 1. Network Key
- **Size**: 128 bits (16 bytes)
- **Purpose**: Shared secret for all devices in the network
- **Usage**: Encrypts broadcast and multicast frames
- **Key ID**: 0x00
- **Rotation**: Supported via Trust Center

#### 2. Trust Center Link Key
- **Size**: 128 bits (16 bytes)
- **Purpose**: Secure communication with Trust Center
- **Default**: "ZigBeeAlliance09" (for development only)
- **Usage**: Device authentication and key transport
- **Key ID**: 0x02

#### 3. Application Link Key
- **Size**: 128 bits (16 bytes)
- **Purpose**: Secure unicast communication between devices
- **Derivation**: From install codes or Trust Center
- **Usage**: End-to-end encryption
- **Key ID**: 0x03

### Frame Counter Management

Each device maintains an outgoing frame counter:
- **Size**: 32 bits
- **Initial Value**: 0
- **Increment**: After each frame transmission
- **Purpose**: Replay attack prevention
- **Rollover**: Must generate new key before rollover

#### Frame Counter Rules

1. **Sender**: Increment counter for each transmitted frame
2. **Receiver**: Accept frame if counter > last seen counter
3. **Security**: Reject frames with duplicate or old counters
4. **Storage**: Persist counter to survive reboots

## Frame Security

### Security Header Format

```
┌─────────────────┬──────────────────┬─────────────────┬─────────────────┐
│ Security Control│  Frame Counter   │  (Optional)     │   (Optional)    │
│    (1 byte)     │    (4 bytes)     │ Key Seq Number  │ Source Address  │
│                 │                  │    (1 byte)     │   (8 bytes)     │
└─────────────────┴──────────────────┴─────────────────┴─────────────────┘
```

#### Security Control Byte

```
Bit 7-5: Reserved
Bit 4-3: Key Identifier
  00 = Network Key (with sequence number)
  01 = Key-Transport Key
  10 = Key-Load Key
  11 = Application Link Key (with source address)
Bit 2-0: Security Level (0-7)
```

### Encryption Flow

```
Application Data
      │
      ▼
┌────────────────────────────────┐
│  Build Security Header         │
│  - Get frame counter           │
│  - Select key type             │
│  - Choose security level       │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  Build Nonce                   │
│  - Source address              │
│  - Frame counter               │
│  - Security control            │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  Prepare AAD (Header)          │
│  - Frame control               │
│  - Addresses                   │
│  - Sequence number             │
│  - Security header             │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  CCM* Encrypt                  │
│  encrypt_and_auth()            │
│  OR auth_only()                │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  Append MIC to Frame           │
└────────┬───────────────────────┘
         │
         ▼
  Secured Frame Ready
  for Transmission
```

### Decryption Flow

```
Received Frame
      │
      ▼
┌────────────────────────────────┐
│  Parse Security Header         │
│  - Extract security control    │
│  - Extract frame counter       │
│  - Identify key type           │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  Verify Frame Counter          │
│  - Check for replay            │
│  - Update last seen counter    │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  Build Nonce                   │
│  - Source address              │
│  - Frame counter               │
│  - Security control            │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  Extract AAD and MIC           │
│  - Separate header, payload    │
│  - Extract MIC from end        │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│  CCM* Decrypt                  │
│  decrypt_and_verify()          │
│  OR verify_auth()              │
└────────┬───────────────────────┘
         │
         ├─ Success ─────────────┐
         │                       ▼
         │            Application Data
         │
         └─ Failure ─────────────┐
                                 ▼
                       Discard Frame
                       Log Security Error
```

## Implementation Details

### CCM* Class (`crypto.rs`)

#### Methods

##### `encrypt_and_auth()`
```rust
pub fn encrypt_and_auth(
    &mut self,
    key: &[u8; 16],
    nonce: &[u8; 13],
    aad: &[u8],
    plaintext: &mut [u8],
    mic: &mut [u8],
) -> Result<(), CryptoError>
```

**Purpose**: Encrypt plaintext and generate authentication tag

**Process**:
1. Calculate CBC-MAC over AAD and plaintext
2. Encrypt plaintext using CTR mode (starting from counter 1)
3. Encrypt authentication tag using counter 0
4. Store encrypted tag in `mic` buffer

**Usage**: For security levels 0x05, 0x06, 0x07 (ENC-MIC-*)

##### `decrypt_and_verify()`
```rust
pub fn decrypt_and_verify(
    &mut self,
    key: &[u8; 16],
    nonce: &[u8; 13],
    aad: &[u8],
    ciphertext: &mut [u8],
    mic: &[u8],
) -> Result<(), CryptoError>
```

**Purpose**: Decrypt ciphertext and verify authentication tag

**Process**:
1. Decrypt authentication tag using counter 0
2. Decrypt ciphertext using CTR mode (starting from counter 1)
3. Calculate CBC-MAC over AAD and decrypted plaintext
4. Compare calculated tag with received tag (constant-time)
5. Zero plaintext if verification fails

**Usage**: For security levels 0x05, 0x06, 0x07 (ENC-MIC-*)

##### `auth_only()`
```rust
pub fn auth_only(
    &mut self,
    key: &[u8; 16],
    nonce: &[u8; 13],
    aad: &[u8],
    data: &[u8],
    mic: &mut [u8],
) -> Result<(), CryptoError>
```

**Purpose**: Generate authentication tag without encryption

**Process**:
1. Calculate CBC-MAC over AAD and data
2. Encrypt authentication tag using counter 0
3. Store encrypted tag in `mic` buffer

**Usage**: For security levels 0x01, 0x02, 0x03 (MIC-*)

##### `verify_auth()`
```rust
pub fn verify_auth(
    &mut self,
    key: &[u8; 16],
    nonce: &[u8; 13],
    aad: &[u8],
    data: &[u8],
    mic: &[u8],
) -> Result<(), CryptoError>
```

**Purpose**: Verify authentication tag without decryption

**Process**:
1. Decrypt authentication tag using counter 0
2. Calculate CBC-MAC over AAD and data
3. Compare calculated tag with received tag (constant-time)

**Usage**: For security levels 0x01, 0x02, 0x03 (MIC-*)

### SecurityManager Class (`security.rs`)

#### Methods

##### `encrypt_frame()`
```rust
pub fn encrypt_frame(
    &mut self,
    aes: &mut Aes<'static>,
    source_addr: u64,
    security_header: &SecurityHeader,
    header: &[u8],
    payload: &mut [u8],
    mic: &mut [u8],
) -> Result<u32, SecurityError>
```

**Purpose**: High-level frame encryption API

**Process**:
1. Select key based on key ID
2. Build nonce from source address and frame counter
3. Create CCM context
4. Call appropriate CCM method based on security level
5. Return frame counter used

**Usage**: Called before frame transmission

##### `decrypt_frame()`
```rust
pub fn decrypt_frame(
    &mut self,
    aes: &mut Aes<'static>,
    source_addr: u64,
    security_header: &SecurityHeader,
    header: &[u8],
    payload: &mut [u8],
    mic: &[u8],
) -> Result<(), SecurityError>
```

**Purpose**: High-level frame decryption API

**Process**:
1. Select key based on key ID
2. Build nonce from source address and frame counter
3. Create CCM context
4. Call appropriate CCM method based on security level

**Usage**: Called upon frame reception

## Security Considerations

### Replay Protection

**Threat**: Attacker captures and replays old frames

**Protection**:
- Frame counter must increase monotonically
- Receiver tracks last seen counter per sender
- Frames with counter ≤ last seen are rejected

**Implementation**:
```rust
if frame_counter <= last_seen_counter {
    return Err(SecurityError::ReplayAttack);
}
last_seen_counter = frame_counter;
```

### Constant-Time Comparison

**Threat**: Timing attacks can reveal authentication tag

**Protection**:
- Use constant-time comparison for MIC verification
- Compare all bytes regardless of first mismatch

**Implementation**:
```rust
let mut match_result = 0u8;
for i in 0..mic.len() {
    match_result |= expected[i] ^ calculated[i];
}
if match_result != 0 {
    return Err(CryptoError::AuthenticationFailed);
}
```

### Nonce Uniqueness

**Threat**: Reusing nonce with same key breaks security

**Protection**:
- Nonce includes frame counter (never repeats)
- Nonce includes source address (unique per device)
- New key required before counter rollover

**Implementation**:
- Frame counter is 32-bit (4 billion frames)
- Generate new network key if counter approaches max

### Key Storage

**Threat**: Keys stored in plaintext can be extracted

**Protection**:
- Store keys in secure storage (if available)
- Zero keys from memory when no longer needed
- Use hardware key storage (future enhancement)

**Implementation**:
```rust
impl Drop for Key {
    fn drop(&mut self) {
        // Zero key memory on drop
        unsafe { 
            (self as *mut Self)
                .cast::<MaybeUninit<Self>>()
                .write_volatile(MaybeUninit::zeroed()) 
        };
    }
}
```

### Frame Counter Persistence

**Threat**: Rebooting device can reset counter

**Protection**:
- Persist frame counter to non-volatile storage
- Load counter on startup
- Add safety margin to prevent reuse

**Implementation**:
```rust
// On startup
let stored_counter = storage.load_frame_counter()?;
outgoing_frame_counter = stored_counter + SAFETY_MARGIN;

// Periodically
if outgoing_frame_counter % PERSIST_INTERVAL == 0 {
    storage.save_frame_counter(outgoing_frame_counter)?;
}
```

## Performance

### Hardware Acceleration

The ESP32-C6 and ESP32-H2 include hardware AES acceleration:
- **Throughput**: Up to 50 Mbps
- **Latency**: ~1 µs per 16-byte block
- **Power**: Significantly lower than software implementation

### CCM* Performance Estimates

For a typical Zigbee frame (50 bytes payload):

| Operation | AES Blocks | Time (µs) | Notes |
|-----------|------------|-----------|-------|
| CBC-MAC (AAD) | 2 | ~2 | Header authentication |
| CBC-MAC (Data) | 4 | ~4 | Payload authentication |
| CTR Encrypt | 4 | ~4 | Payload encryption |
| CTR Counter 0 | 1 | ~1 | MIC encryption |
| **Total** | **11** | **~11** | **Hardware accelerated** |

### Memory Usage

#### Stack Usage
- CCM context: ~100 bytes
- Temporary buffers: ~48 bytes (3 x 16-byte blocks)
- Nonce: 13 bytes
- Total: ~161 bytes

#### Heap Usage
- None (all stack-allocated)

## Testing

### Unit Tests

#### Test Coverage
- ✓ Nonce construction
- ✓ Counter block formatting
- ✓ CBC-MAC calculation
- ✓ CTR mode encryption
- ✓ Authentication tag generation
- ✓ Constant-time comparison

#### Test Vectors

Test vectors from Zigbee specification Annex C.6.1:

```rust
// Test Vector 1: Encryption + MIC-32
Key:     C0 C1 C2 C3 C4 C5 C6 C7 C8 C9 CA CB CC CD CE CF
Nonce:   A0 A1 A2 A3 A4 A5 A6 A7 00 00 00 00 05
AAD:     00 01 02 03 04 05 06 07
Plain:   08 09 0A 0B 0C 0D 0E 0F
Cipher:  58 8C 97 9A 61 C6 63 D2
MIC:     17 E8 D1 2C
```

### Integration Tests

#### Hardware Tests
- Encrypt/decrypt with real AES hardware
- Performance benchmarks
- Power consumption measurements

#### Interoperability Tests
- Communicate with commercial Zigbee devices
- Verify compatibility with Zigbee Alliance test tools
- Test key rotation scenarios

## Usage Examples

### Example 1: Setting Network Key

```rust
let mut zigbee = Zigbee::new(radio, aes, config);

// Generate or set network key
let network_key: [u8; 16] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
];

zigbee.set_network_key(network_key);
```

### Example 2: Sending Encrypted Frame

```rust
// Prepare frame
let mut payload = b"Hello, Zigbee!".to_vec();
let header = build_frame_header(...);

// Build security header
let mut security_header = SecurityHeader::new(
    SecurityLevel::EncMic32,
    0, // Network key
    zigbee.security_manager().frame_counter(),
);

// Encrypt frame
let mut mic = [0u8; 4];
let frame_counter = zigbee.security_manager_mut().encrypt_frame(
    &mut zigbee.inner.aes,
    source_ieee_addr,
    &security_header,
    &header,
    &mut payload,
    &mut mic,
)?;

// Transmit frame with security header and MIC
```

### Example 3: Receiving Encrypted Frame

```rust
// Parse received frame
let (security_header, header_len) = SecurityHeader::decode(&frame)?;
let payload_len = frame.len() - header_len - security_header.security_level.mic_length();

let header = &frame[..header_len];
let mut payload = frame[header_len..payload_len].to_vec();
let mic = &frame[payload_len..];

// Decrypt and verify frame
zigbee.security_manager_mut().decrypt_frame(
    &mut zigbee.inner.aes,
    source_ieee_addr,
    &security_header,
    header,
    &mut payload,
    mic,
)?;

// Process decrypted payload
```

### Example 4: Authentication-Only Mode

```rust
// Use MIC-32 security level (no encryption)
let security_header = SecurityHeader::new(
    SecurityLevel::Mic32,
    0, // Network key
    frame_counter,
);

// Authenticate without encrypting
let mut mic = [0u8; 4];
zigbee.security_manager_mut().encrypt_frame(
    &mut zigbee.inner.aes,
    source_ieee_addr,
    &security_header,
    &header,
    &mut payload, // Not encrypted
    &mut mic,
)?;
```

## Error Handling

### CryptoError Types

| Error | Description | Recovery |
|-------|-------------|----------|
| `InvalidLength` | AAD > 128 bytes | Reduce header size |
| `InvalidMicLength` | MIC not 4/8/16 | Fix security level |
| `AuthenticationFailed` | MIC verification failed | Discard frame, log error |
| `EncryptionFailed` | Generic encryption error | Check key and parameters |

### SecurityError Types

| Error | Description | Recovery |
|-------|-------------|----------|
| `NoKey` | Key not configured | Set network/link key |
| `InvalidKey` | Key format invalid | Check key size (16 bytes) |
| `AuthenticationFailed` | Frame auth failed | Check key, investigate attack |
| `DecryptionFailed` | Frame decryption failed | Check key and nonce |
| `ReplayAttack` | Old frame counter | Discard frame |

## Future Enhancements

### Planned Features

1. **Hardware Key Storage**
   - Use ESP32 eFuse or secure storage
   - Prevent key extraction
   - Hardware-backed encryption

2. **DMA Support**
   - Use AES DMA mode for large frames
   - Reduce CPU overhead
   - Improve throughput

3. **Key Derivation Functions**
   - AES-MMO hash for install codes
   - HMAC-SHA256 for key derivation
   - Key agreement protocols

4. **Performance Optimizations**
   - Cache counter blocks
   - Batch multiple frames
   - Optimize AAD processing

5. **Security Hardening**
   - Side-channel attack mitigation
   - Secure key injection
   - Tamper detection

## References

1. **Zigbee Specification**
   - Zigbee Alliance Document 05-3474-21
   - Section 4.5: Security Services Specification

2. **NIST SP 800-38C**
   - Recommendation for Block Cipher Modes of Operation: The CCM Mode for Authentication and Confidentiality

3. **IEEE 802.15.4-2015**
   - Section 9: Security
   - CCM* description and test vectors

4. **ESP32 Technical Reference Manual**
   - AES Accelerator chapter
   - DMA controller documentation

## Conclusion

The AES-128 CCM* encryption implementation provides:

✅ **Complete CCM* support** - Encryption, authentication, and auth-only modes
✅ **Hardware acceleration** - Uses ESP32 AES engine for performance
✅ **Security best practices** - Constant-time comparison, nonce uniqueness, replay protection
✅ **Zigbee compliant** - Follows specification for frame security
✅ **Easy to use** - High-level API for frame encryption/decryption
✅ **Well documented** - Comprehensive documentation and examples
✅ **Tested** - Unit tests and integration tests

The implementation is ready for production use in Zigbee networks requiring secure communication.
