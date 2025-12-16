// apps/playground/src/config/firebase.ts
/**
 * Firebase configuration
 *
 * Uses environment variables if available, otherwise defaults to production config.
 * These config values are safe to expose in the client (public by design).
 */

export function getFirebaseConfig() {
  // Use environment variables if provided, otherwise use default production config
  const apiKey = import.meta.env.VITE_FIREBASE_API_KEY || "AIzaSyBd-SISBmH74YrJEtV8AVHkttjCPaCt4TI";
  const authDomain = import.meta.env.VITE_FIREBASE_AUTH_DOMAIN || "sruja-ai.firebaseapp.com";
  const projectId = import.meta.env.VITE_FIREBASE_PROJECT_ID || "sruja-ai";
  const storageBucket =
    import.meta.env.VITE_FIREBASE_STORAGE_BUCKET || "sruja-ai.firebasestorage.app";
  const messagingSenderId = import.meta.env.VITE_FIREBASE_MESSAGING_SENDER_ID || "278447771500";
  const appId = import.meta.env.VITE_FIREBASE_APP_ID || "1:278447771500:web:bc9d58d71ebb32b855ad49";

  return {
    apiKey,
    authDomain,
    projectId,
    storageBucket,
    messagingSenderId,
    appId,
  };
}
