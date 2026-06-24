import { TipButton } from "./TipButton";
import type { useVault } from "../hooks/useVault";

type VaultApi = ReturnType<typeof useVault>;

/** Vault-X lock screen, credential table, and add-secret form. */
export function VaultPanel({ vault }: { vault: VaultApi }) {
  const {
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
    handleInitVault,
    handleUnlockVault,
    handleLockVault,
    handleAddSecret,
    handleDeleteSecret,
    handleCopySecret,
    handleToggleReveal,
  } = vault;

  if (!isVaultUnlocked) {
    return (
      <div className="vault-panel">
        <div className="lock-screen">
          <div className="lock-icon">🔒</div>
          <h2 style={{ marginBottom: "8px", fontSize: "16px", fontWeight: "600" }}>Vault-X Encrypted Secrets</h2>
          <p style={{ color: "var(--text-muted)", fontSize: "12px", marginBottom: "20px", textAlign: "center", maxWidth: "250px" }}>
            {isVaultInitialized
              ? "Enter your master key credentials to unlock safe data."
              : "Initialize a new secure AES-256 vault by setting a master key password."}
          </p>
          <div className="vault-input-group">
            <input
              type="password"
              className="vault-password-input"
              placeholder="Enter Master Password"
              value={vaultPassword}
              onChange={(e) => setVaultPassword(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  isVaultInitialized ? void handleUnlockVault() : void handleInitVault();
                }
              }}
            />
            <button
              className="action-btn accent has-tooltip"
              style={{ padding: "10px", justifyContent: "center" }}
              data-tooltip={
                isVaultInitialized
                  ? "Unlock your encrypted secrets with your master password"
                  : "Create a new encrypted vault with a master password"
              }
              onClick={() => (isVaultInitialized ? void handleUnlockVault() : void handleInitVault())}
            >
              {isVaultInitialized ? "Unlock Vault" : "Initialize Vault"}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="vault-panel">
      <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
          <div>
            <h2 style={{ fontSize: "18px", fontWeight: "600" }}>Vault-X Credentials Registry</h2>
            <p style={{ color: "var(--text-muted)", fontSize: "12px" }}>
              Highly secure offline keys, tokens, and SSH configurations
            </p>
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <TipButton
              className="action-btn"
              tip="Add an API key, token, or password to the encrypted vault"
              onClick={() => setShowAddSecret(true)}
            >
              + Add Secret Key
            </TipButton>
            <TipButton className="action-btn" tip="Lock the vault and clear secrets from memory" onClick={() => void handleLockVault()}>
              Lock Vault
            </TipButton>
          </div>
        </div>

        {showAddSecret && (
          <div
            style={{
              background: "rgba(255,255,255,0.03)",
              padding: "16px",
              borderRadius: "8px",
              border: "1px solid var(--border-color)",
              marginBottom: "16px",
            }}
          >
            <h3 style={{ fontSize: "13px", fontWeight: "600", marginBottom: "12px" }}>Register Secure Credential</h3>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "12px", marginBottom: "12px" }}>
              <input
                type="text"
                placeholder="Key Name (e.g. OPENAI_KEY)"
                className="search-input"
                value={newSecretName}
                onChange={(e) => setNewSecretName(e.target.value)}
              />
              <input
                type="text"
                placeholder="Project Mapping Path (Optional)"
                className="search-input"
                value={newSecretPath}
                onChange={(e) => setNewSecretPath(e.target.value)}
              />
              <input
                type="password"
                placeholder="Secret Value / API Token"
                className="search-input"
                value={newSecretValue}
                onChange={(e) => setNewSecretValue(e.target.value)}
              />
            </div>
            <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
              <TipButton className="action-btn" tip="Close the add-secret form without saving" onClick={() => setShowAddSecret(false)}>
                Cancel
              </TipButton>
              <TipButton className="action-btn accent" tip="Encrypt and store this credential in the vault" onClick={() => void handleAddSecret()}>
                Save Securely
              </TipButton>
            </div>
          </div>
        )}

        <div className="vault-table-container">
          <table className="vault-table">
            <thead>
              <tr>
                <th>Secret Name</th>
                <th>Project Mapping Path</th>
                <th>Token / Key Value</th>
                <th style={{ width: "120px" }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {secretMetadata.map((secret) => (
                <tr key={`${secret.name}-${secret.index}`}>
                  <td style={{ fontWeight: "600" }}>{secret.name}</td>
                  <td style={{ fontStyle: "italic", color: "var(--text-muted)", fontSize: "11px" }}>
                    {secret.path || "Global / Unmapped"}
                  </td>
                  <td
                    className="key-mask has-tooltip"
                    data-tooltip="Click to copy this secret to your clipboard"
                    onClick={() => void handleCopySecret(secret.index, secret.name)}
                  >
                    {revealedIndex === secret.index ? revealedValueRef.current : "•••••••••••••••• (Click to Copy)"}
                  </td>
                  <td>
                    <div style={{ display: "flex", gap: "8px" }}>
                      <TipButton
                        className="tab-btn"
                        style={{ padding: "2px 6px" }}
                        tip={revealedIndex === secret.index ? "Hide the secret value on screen" : "Temporarily show the secret value on screen"}
                        onClick={() => void handleToggleReveal(secret.index)}
                      >
                        {revealedIndex === secret.index ? "Hide" : "Show"}
                      </TipButton>
                      <TipButton
                        className="tab-btn"
                        style={{ padding: "2px 6px", color: "#ef4444" }}
                        tip="Remove this secret from the vault permanently"
                        onClick={() => void handleDeleteSecret(secret.index)}
                      >
                        Delete
                      </TipButton>
                    </div>
                  </td>
                </tr>
              ))}
              {secretMetadata.length === 0 && (
                <tr>
                  <td colSpan={4} style={{ textAlign: "center", color: "var(--text-muted)", padding: "24px" }}>
                    No secrets registered yet. Click "+ Add Secret Key" to secure your first credential!
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
