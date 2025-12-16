// apps/playground/src/utils/cryptoService.ts

/**
 * Client-side encryption service using Web Crypto API
 * Implements AES-256-GCM as specified
 */
export class CryptoService {
  private readonly algorithm = "AES-GCM";
  private readonly keyLength = 256; // bits
  private readonly ivLength = 12; // bytes for GCM

  /**
   * Generate a random encryption key (32 bytes for AES-256)
   */
  async generateKey(): Promise<globalThis.CryptoKey> {
    return crypto.subtle.generateKey(
      {
        name: this.algorithm,
        length: this.keyLength,
      },
      true, // extractable
      ["encrypt", "decrypt"]
    );
  }

  /**
   * Export key to Base64 string for URL storage
   */
  async exportKey(key: globalThis.CryptoKey): Promise<string> {
    const exported = await crypto.subtle.exportKey("raw", key);
    const bytes = new Uint8Array(exported);
    return this.arrayBufferToBase64(bytes);
  }

  /**
   * Import key from Base64 string
   */
  async importKey(keyBase64: string): Promise<globalThis.CryptoKey> {
    const bytes = this.base64ToArrayBuffer(keyBase64);
    return crypto.subtle.importKey(
      "raw",
      bytes as BufferSource,
      {
        name: this.algorithm,
        length: this.keyLength,
      },
      true, // extractable
      ["encrypt", "decrypt"]
    );
  }

  /**
   * Generate random IV (12 bytes for GCM)
   */
  private generateIV(): Uint8Array {
    return crypto.getRandomValues(new Uint8Array(this.ivLength));
  }

  /**
   * Encrypt plaintext DSL using AES-GCM
   * @returns Object with ciphertext and IV (both Base64)
   */
  async encrypt(
    plaintext: string,
    key: globalThis.CryptoKey
  ): Promise<{ ciphertext: string; iv: string }> {
    const iv = this.generateIV();
    const encoder = new TextEncoder();
    const data = encoder.encode(plaintext);

    const encrypted = await crypto.subtle.encrypt(
      {
        name: this.algorithm,
        iv: iv as unknown as BufferSource,
      },
      key,
      data as unknown as BufferSource
    );

    return {
      ciphertext: this.arrayBufferToBase64(new Uint8Array(encrypted)),
      iv: this.arrayBufferToBase64(iv),
    };
  }

  /**
   * Decrypt ciphertext using AES-GCM
   */
  async decrypt(ciphertext: string, iv: string, key: globalThis.CryptoKey): Promise<string> {
    const ciphertextBytes = this.base64ToArrayBuffer(ciphertext);
    const ivBytes = this.base64ToArrayBuffer(iv);

    const decrypted = await crypto.subtle.decrypt(
      {
        name: this.algorithm,
        iv: ivBytes as unknown as BufferSource,
      },
      key,
      ciphertextBytes as unknown as BufferSource
    );

    const decoder = new TextDecoder();
    return decoder.decode(decrypted);
  }

  /**
   * Convert ArrayBuffer/Uint8Array to Base64
   */
  private arrayBufferToBase64(buffer: Uint8Array): string {
    const binary = String.fromCharCode(...buffer);
    return btoa(binary);
  }

  /**
   * Convert Base64 to ArrayBuffer
   */
  private base64ToArrayBuffer(base64: string): Uint8Array {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }
}

// Singleton instance
export const cryptoService = new CryptoService();
