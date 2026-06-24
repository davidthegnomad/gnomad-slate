import { useCallback, useRef, useState } from "react";
import { invoke } from "../utils/ipc";
import { invokeErrorMessage } from "../utils/invokeError";
import type { SecretMeta } from "../types/app";

type ShowMessage = (message: string) => void;

/**
 * Vault-X UI state and IPC handlers.
 *
 * Secret values are kept in `revealedValueRef` (not React state) so plaintext
 * does not appear in React DevTools snapshots. Only the revealed row index
 * is stored in state to trigger re-renders.
 */
export function useVault(showAppMessage: ShowMessage) {
  const [isVaultInitialized, setIsVaultInitialized] = useState(false);
  const [isVaultUnlocked, setIsVaultUnlocked] = useState(false);
  const [vaultPassword, setVaultPassword] = useState("");
  const [secretMetadata, setSecretMetadata] = useState<SecretMeta[]>([]);
  const [revealedIndex, setRevealedIndex] = useState<number | null>(null);
  const revealedValueRef = useRef("");

  const [showAddSecret, setShowAddSecret] = useState(false);
  const [newSecretName, setNewSecretName] = useState("");
  const [newSecretPath, setNewSecretPath] = useState("");
  const [newSecretValue, setNewSecretValue] = useState("");

  const clearRevealedSecret = useCallback(() => {
    setRevealedIndex(null);
    revealedValueRef.current = "";
  }, []);

  const loadSecretMetadata = useCallback(async () => {
    try {
      const meta = await invoke<SecretMeta[]>("list_secret_metadata");
      setSecretMetadata(meta);
    } catch (e) {
      console.error("Failed to load secret metadata:", e);
      setSecretMetadata([]);
    }
  }, []);

  const checkVaultStatus = useCallback(async () => {
    try {
      const initialized = await invoke<boolean>("is_vault_initialized");
      setIsVaultInitialized(initialized);
      const unlocked = await invoke<boolean>("is_vault_unlocked");
      setIsVaultUnlocked(unlocked);
      if (unlocked) {
        await loadSecretMetadata();
      }
    } catch (e) {
      console.error("Failed to check vault status:", e);
    }
  }, [loadSecretMetadata]);

  const handleInitVault = useCallback(async () => {
    if (!vaultPassword) return;
    try {
      await invoke("initialize_vault", { password: vaultPassword });
      setIsVaultInitialized(true);
      setIsVaultUnlocked(true);
      setVaultPassword("");
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      showAppMessage("Failed to initialize: " + invokeErrorMessage(e));
    }
  }, [vaultPassword, clearRevealedSecret, loadSecretMetadata, showAppMessage]);

  const handleUnlockVault = useCallback(async () => {
    if (!vaultPassword) return;
    try {
      await invoke("unlock_vault", { password: vaultPassword });
      setIsVaultUnlocked(true);
      setVaultPassword("");
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      showAppMessage(invokeErrorMessage(e));
    }
  }, [vaultPassword, clearRevealedSecret, loadSecretMetadata, showAppMessage]);

  const handleLockVault = useCallback(async () => {
    await invoke("lock_vault");
    setIsVaultUnlocked(false);
    setVaultPassword("");
    setSecretMetadata([]);
    clearRevealedSecret();
  }, [clearRevealedSecret]);

  const handleAddSecret = useCallback(async () => {
    if (!newSecretName || !newSecretValue) return;
    try {
      await invoke("add_secret", {
        name: newSecretName,
        path: newSecretPath,
        value: newSecretValue,
      });
      setNewSecretName("");
      setNewSecretPath("");
      setNewSecretValue("");
      setShowAddSecret(false);
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      showAppMessage("Failed to add secret: " + invokeErrorMessage(e));
    }
  }, [newSecretName, newSecretPath, newSecretValue, clearRevealedSecret, loadSecretMetadata, showAppMessage]);

  const handleDeleteSecret = useCallback(async (index: number) => {
    try {
      await invoke("delete_secret", { index });
      clearRevealedSecret();
      await loadSecretMetadata();
    } catch (e: unknown) {
      showAppMessage("Failed to delete secret: " + invokeErrorMessage(e));
    }
  }, [clearRevealedSecret, loadSecretMetadata, showAppMessage]);

  const handleCopySecret = useCallback(async (index: number, name: string) => {
    try {
      await invoke("copy_secret_to_clipboard", { index });
      showAppMessage(`${name} copied to clipboard!`);
    } catch (e: unknown) {
      showAppMessage("Copy failed: " + invokeErrorMessage(e));
    }
  }, [showAppMessage]);

  const handleToggleReveal = useCallback(async (index: number) => {
    if (revealedIndex === index) {
      clearRevealedSecret();
      return;
    }
    try {
      const value = await invoke<string>("reveal_secret_value", { index });
      revealedValueRef.current = value;
      setRevealedIndex(index);
    } catch (e: unknown) {
      showAppMessage("Could not reveal secret: " + invokeErrorMessage(e));
    }
  }, [revealedIndex, clearRevealedSecret, showAppMessage]);

  return {
    isVaultInitialized,
    isVaultUnlocked,
    vaultPassword,
    setVaultPassword,
    secretMetadata,
    revealedIndex,
    revealedValueRef,
    showAddSecret,
    setShowAddSecret,
    newSecretName,
    setNewSecretName,
    newSecretPath,
    setNewSecretPath,
    newSecretValue,
    setNewSecretValue,
    clearRevealedSecret,
    checkVaultStatus,
    loadSecretMetadata,
    handleInitVault,
    handleUnlockVault,
    handleLockVault,
    handleAddSecret,
    handleDeleteSecret,
    handleCopySecret,
    handleToggleReveal,
  };
}
