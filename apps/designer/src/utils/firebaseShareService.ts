// apps/playground/src/utils/firebaseShareService.ts
// @ts-ignore - Firebase types may not be fully available
import { initializeApp, type FirebaseApp } from "firebase/app";
// @ts-ignore
import {
  getFirestore,
  doc,
  getDoc,
  setDoc,
  onSnapshot,
  serverTimestamp,
  type Firestore,
} from "firebase/firestore";
import { cryptoService } from "./cryptoService";

const PROJECT_ID_LENGTH = 21; // Firestore document ID length

/**
 * Generate a random project ID (Firestore-compatible)
 */
function generateProjectId(): string {
  // Generate URL-safe random string
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";
  for (let i = 0; i < PROJECT_ID_LENGTH; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Firebase project document schema
 */
interface ProjectDocument {
  ciphertext: string;
  iv: string;
  version: number;
  updatedAt: any; // serverTimestamp
}

/**
 * Encrypted share service using Firebase + client-side encryption
 * Implements the V1 spec: anonymous, encrypted, URL-based access
 */
export class FirebaseShareService {
  private app: FirebaseApp | null = null;
  private db: Firestore | null = null;
  private currentKey: globalThis.CryptoKey | null = null;
  private currentProjectId: string | null = null;

  /**
   * Initialize Firebase (call once on app startup)
   */
  async initialize(config: {
    apiKey: string;
    authDomain: string;
    projectId: string;
    storageBucket: string;
    messagingSenderId: string;
    appId: string;
  }): Promise<void> {
    if (this.app) return; // Already initialized

    this.app = initializeApp(config);
    this.db = getFirestore(this.app);
  }

  /**
   * Check if Firebase is initialized
   */
  private ensureInitialized(): void {
    if (!this.db) {
      throw new Error("Firebase not initialized. Call initialize() first.");
    }
  }

  /**
   * Generate new project ID and encryption key
   */
  async createNewProject(): Promise<{
    projectId: string;
    keyBase64: string;
    key: globalThis.CryptoKey;
  }> {
    const projectId = generateProjectId();
    const key = await cryptoService.generateKey();
    const keyBase64 = await cryptoService.exportKey(key);

    this.currentProjectId = projectId;
    this.currentKey = key;

    return { projectId, keyBase64, key };
  }

  /**
   * Set current project from URL
   */
  async setProjectFromUrl(projectId: string, keyBase64: string): Promise<void> {
    try {
      const key = await cryptoService.importKey(keyBase64);
      this.currentProjectId = projectId;
      this.currentKey = key;
    } catch (error) {
      throw new Error("Invalid encryption key");
    }
  }

  /**
   * Save DSL to Firebase (encrypted)
   */
  async saveProject(dsl: string): Promise<void> {
    this.ensureInitialized();
    if (!this.currentProjectId || !this.currentKey) {
      throw new Error("No active project. Create or load a project first.");
    }

    // Encrypt DSL
    const { ciphertext, iv } = await cryptoService.encrypt(dsl, this.currentKey);

    // Save to Firestore
    const projectRef = doc(this.db!, "projects", this.currentProjectId);
    await setDoc(
      projectRef,
      {
        ciphertext,
        iv,
        version: 1,
        updatedAt: serverTimestamp(),
      } as ProjectDocument,
      { merge: false } // Overwrite entire document (last-write-wins)
    );
  }

  /**
   * Load DSL from Firebase (decrypted)
   */
  async loadProject(projectId: string, keyBase64: string): Promise<string> {
    this.ensureInitialized();

    // Import key
    const key = await cryptoService.importKey(keyBase64);

    // Fetch from Firestore
    const projectRef = doc(this.db!, "projects", projectId);
    const snapshot = await getDoc(projectRef);

    if (!snapshot.exists()) {
      throw new Error("Project not found");
    }

    const data = snapshot.data() as ProjectDocument;

    // Decrypt
    try {
      const dsl = await cryptoService.decrypt(data.ciphertext, data.iv, key);
      this.currentProjectId = projectId;
      this.currentKey = key;
      return dsl;
    } catch (error) {
      throw new Error("Cannot decrypt project. Invalid key or corrupted data.");
    }
  }

  /**
   * Load project with real-time sync (using onSnapshot)
   * Returns unsubscribe function and callback for updates
   *
   * @param projectId Project ID
   * @param keyBase64 Base64-encoded encryption key
   * @param onUpdate Callback when project updates (receives decrypted DSL)
   * @returns Unsubscribe function to stop listening
   */
  loadProjectRealtime(
    projectId: string,
    keyBase64: string,
    onUpdate: (dsl: string) => void
  ): () => void {
    this.ensureInitialized();

    const projectRef = doc(this.db!, "projects", projectId);

    // Set current project state
    cryptoService
      .importKey(keyBase64)
      .then((key) => {
        this.currentProjectId = projectId;
        this.currentKey = key;
      })
      .catch((err) => {
        console.error("Failed to import key:", err);
      });

    // Set up real-time listener
    const unsubscribe = onSnapshot(
      projectRef,
      async (docSnap) => {
        if (!docSnap.exists()) {
          console.warn(`Project ${projectId} not found`);
          return;
        }

        try {
          const data = docSnap.data() as ProjectDocument;

          // Import the key (expects Base64 string)
          const cryptoKey = await cryptoService.importKey(keyBase64);

          // Decrypt the DSL
          const plaintext = await cryptoService.decrypt(data.ciphertext, data.iv, cryptoKey);

          // Update current state
          this.currentProjectId = projectId;
          this.currentKey = cryptoKey;

          // Call the update callback
          onUpdate(plaintext);
        } catch (err) {
          console.error("Failed to decrypt project update:", err);
          // Don't throw - let the listener continue
        }
      },
      (err) => {
        console.error("Firestore listener error:", err);
      }
    );

    return unsubscribe;
  }

  /**
   * Get current project ID
   */
  getCurrentProjectId(): string | null {
    return this.currentProjectId;
  }

  /**
   * Get current encryption key as Base64 (for URL)
   */
  async getCurrentKeyBase64(): Promise<string | null> {
    if (!this.currentKey) return null;
    return await cryptoService.exportKey(this.currentKey);
  }

  /**
   * Build share URL
   * Format: /designer/{projectId}#k={keyBase64}
   */
  async buildShareUrl(projectId: string, keyBase64: string, baseUrl?: string): Promise<string> {
    const base = baseUrl || window.location.origin;
    return `${base}/designer/${projectId}#k=${encodeURIComponent(keyBase64)}`;
  }

  /**
   * Parse project ID and key from URL
   * Supports format: /designer/{projectId}#k={keyBase64}
   */
  parseUrl(url: string | URL): { projectId: string | null; keyBase64: string | null } {
    const urlObj = typeof url === "string" ? new URL(url) : url;

    // Extract projectId from path: /designer/{projectId}
    // Also handles root paths like /{projectId} if needed
    const pathMatch = urlObj.pathname.match(/\/(?:designer\/)?([^/]+)$/);
    const projectId = pathMatch ? pathMatch[1] : null;

    // Extract key from fragment: #k={keyBase64}
    const keyMatch = urlObj.hash.match(/[#&]k=([^&]*)/);
    const keyBase64 = keyMatch ? decodeURIComponent(keyMatch[1]) : null;

    return { projectId, keyBase64 };
  }

  /**
   * Check if URL is a project share URL
   */
  isProjectUrl(url: string | URL): boolean {
    const { projectId, keyBase64 } = this.parseUrl(url);
    // Project ID should be at least 10 chars (Firestore ID length) and have a key
    return projectId !== null && projectId.length >= 10 && keyBase64 !== null;
  }
}

// Singleton instance
export const firebaseShareService = new FirebaseShareService();
