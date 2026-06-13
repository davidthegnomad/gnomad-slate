const VAULT_SALT = "com.gnomadstudio.gnomad-slate.vault.v2";
const ITERATIONS = 100000;
const KEY_LEN = 256; // bits

/** Helper to convert string to ArrayBuffer */
function stringToBuffer(str: string): Uint8Array {
  return new TextEncoder().encode(str);
}

/** Helper to convert ArrayBuffer to hex string */
function bufferToHex(buffer: ArrayBuffer): string {
  return Array.from(new Uint8Array(buffer))
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
}

/** Helper to convert hex string to ArrayBuffer */
function hexToBuffer(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

/** Derive a CryptoKey from master password using PBKDF2 */
export async function deriveKey(password: string): Promise<CryptoKey> {
  const baseKey = await crypto.subtle.importKey(
    "raw",
    stringToBuffer(password),
    { name: "PBKDF2" },
    false,
    ["deriveKey", "deriveBits"]
  );

  return crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: stringToBuffer(VAULT_SALT),
      iterations: ITERATIONS,
      hash: "SHA-256",
    },
    baseKey,
    { name: "AES-GCM", length: KEY_LEN },
    true, // exportable for session storage
    ["encrypt", "decrypt"]
  );
}

/** Encrypt payload: returns hex string containing format | salt/nonce | ciphertext */
export async function encryptData(plaintext: string, key: CryptoKey): Promise<string> {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = stringToBuffer(plaintext);

  const ciphertextBuffer = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv: iv,
    },
    key,
    encoded
  );

  const ciphertext = new Uint8Array(ciphertextBuffer);
  
  // Package format: [ 0x02 | 12-byte IV | ciphertext ]
  const packet = new Uint8Array(1 + iv.length + ciphertext.length);
  packet[0] = 0x02; // version v2
  packet.set(iv, 1);
  packet.set(ciphertext, 1 + iv.length);

  return bufferToHex(packet.buffer);
}

/** Decrypt data from format | salt/nonce | ciphertext package */
export async function decryptData(packetHex: string, key: CryptoKey): Promise<string> {
  const packet = hexToBuffer(packetHex);
  if (packet.length < 14 || packet[0] !== 0x02) {
    throw new Error("Invalid cipher packet version");
  }

  const iv = packet.slice(1, 13);
  const ciphertext = packet.slice(13);

  const decryptedBuffer = await crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv: iv,
    },
    key,
    ciphertext
  );

  return new TextDecoder().decode(decryptedBuffer);
}

/** Hash a password for storage verification (PBKDF2-HMAC-SHA256 output) */
export async function hashPasswordForVerification(password: string, saltHex?: string): Promise<{ hash: string; salt: string }> {
  const salt = saltHex ? hexToBuffer(saltHex) : crypto.getRandomValues(new Uint8Array(16));
  
  const baseKey = await crypto.subtle.importKey(
    "raw",
    stringToBuffer(password),
    { name: "PBKDF2" },
    false,
    ["deriveBits"]
  );

  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      salt: salt,
      iterations: ITERATIONS,
      hash: "SHA-256",
    },
    baseKey,
    256 // bits
  );

  return {
    hash: bufferToHex(derivedBits),
    salt: bufferToHex(salt.buffer),
  };
}

/** Verify a password against a stored hash and salt */
export async function verifyPassword(password: string, storedHash: string, saltHex: string): Promise<boolean> {
  const { hash } = await hashPasswordForVerification(password, saltHex);
  
  // Constant time comparison equivalent in JS
  if (hash.length !== storedHash.length) return false;
  let diff = 0;
  for (let i = 0; i < hash.length; i++) {
    diff |= hash.charCodeAt(i) ^ storedHash.charCodeAt(i);
  }
  return diff === 0;
}
