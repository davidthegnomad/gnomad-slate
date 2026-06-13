import React, { useState, useEffect } from "react";
import ReactDOM from "react-dom/client";
import "../../src/App.css";

interface SyncStatus {
  lastSync: number;
  engine: "none" | "chrome" | "gdrive";
  error: string | null;
}

function Options() {
  const [syncEngine, setSyncEngine] = useState<"none" | "chrome" | "gdrive">("none");
  const [encryptNotes, setEncryptNotes] = useState(true);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>({ lastSync: 0, engine: "none", error: null });
  const [isGdriveAuthed, setIsGdriveAuthed] = useState(false);
  const [stats, setStats] = useState({ notesCount: 0, secretsCount: 0, bytesUsed: 0 });

  useEffect(() => {
    // Load current options from storage
    if (typeof chrome !== "undefined" && chrome.storage) {
      chrome.storage.local.get(["syncEngine", "encryptNotes", "syncStatus", "notes", "vault"], (result) => {
        if (result.syncEngine) setSyncEngine(result.syncEngine);
        if (result.encryptNotes !== undefined) setEncryptNotes(result.encryptNotes);
        if (result.syncStatus) setSyncStatus(result.syncStatus);

        // Calculate statistics
        const notes = result.notes || {};
        const vault = result.vault || [];
        const notesCount = Object.keys(notes).length;
        const secretsCount = vault.length;
        
        // Simple serialization length estimate
        const totalStr = JSON.stringify(result);
        setStats({
          notesCount,
          secretsCount,
          bytesUsed: totalStr.length * 2, // rough estimate in bytes
        });
      });

      // Check Google Drive auth status
      chrome.runtime.sendMessage({ type: "check-gdrive-auth" }, (response) => {
        if (response && response.authorized) {
          setIsGdriveAuthed(true);
        }
      });
    }
  }, []);

  const handleSaveSyncEngine = (engine: "none" | "chrome" | "gdrive") => {
    setSyncEngine(engine);
    if (typeof chrome !== "undefined" && chrome.storage) {
      chrome.storage.local.set({ syncEngine: engine }, () => {
        chrome.runtime.sendMessage({ type: "configure-sync", engine });
      });
    }
  };

  const handleToggleEncryption = () => {
    const newValue = !encryptNotes;
    setEncryptNotes(newValue);
    if (typeof chrome !== "undefined" && chrome.storage) {
      chrome.storage.local.set({ encryptNotes: newValue });
    }
  };

  const handleGdriveAuth = () => {
    if (isGdriveAuthed) {
      // Sign out
      chrome.runtime.sendMessage({ type: "gdrive-logout" }, (response) => {
        if (response && response.success) {
          setIsGdriveAuthed(false);
          alert("Successfully signed out of Google Drive.");
        }
      });
    } else {
      // Sign in
      chrome.runtime.sendMessage({ type: "gdrive-login" }, (response) => {
        if (response && response.success) {
          setIsGdriveAuthed(true);
          alert("Successfully authenticated with Google Drive.");
        } else {
          alert("Authentication failed: " + (response?.error || "Unknown error"));
        }
      });
    }
  };

  const triggerManualSync = () => {
    chrome.runtime.sendMessage({ type: "manual-sync" }, (response) => {
      if (response && response.success) {
        alert("Sync completed successfully!");
        setSyncStatus((prev) => ({ ...prev, lastSync: Date.now(), error: null }));
      } else {
        alert("Sync failed: " + (response?.error || "Unknown error"));
        setSyncStatus((prev) => ({ ...prev, error: response?.error || "Sync failed" }));
      }
    });
  };

  return (
    <div className="app-container" style={{ padding: "40px", maxWidth: "800px", margin: "0 auto", overflowY: "auto" }}>
      <header style={{ marginBottom: "30px", borderBottom: "1px solid var(--border-color)", paddingBottom: "20px" }}>
        <h1 className="logo-text" style={{ fontSize: "28px" }}>✦ Gnomad Slate — Settings</h1>
        <p style={{ color: "var(--text-muted)", marginTop: "8px" }}>Configure cross-browser synchronization and local vault options.</p>
      </header>

      <section className="settings-section" style={{ marginBottom: "40px" }}>
        <h2 style={{ fontSize: "20px", marginBottom: "15px", color: "var(--text-normal)" }}>Sync Engine Configuration</h2>
        <p style={{ color: "var(--text-muted)", fontSize: "14px", marginBottom: "20px" }}>
          Choose how you want to keep your notes and vault synchronized across your devices.
        </p>

        <div style={{ display: "grid", gap: "15px", marginBottom: "25px" }}>
          <label style={{
            display: "flex", 
            alignItems: "flex-start", 
            padding: "15px", 
            borderRadius: "8px", 
            border: `1px solid ${syncEngine === "none" ? "var(--accent-color)" : "var(--border-color)"}`, 
            backgroundColor: syncEngine === "none" ? "rgba(168, 85, 247, 0.05)" : "transparent",
            cursor: "pointer"
          }}>
            <input 
              type="radio" 
              name="syncEngine" 
              checked={syncEngine === "none"} 
              onChange={() => handleSaveSyncEngine("none")} 
              style={{ marginTop: "4px", marginRight: "12px" }}
            />
            <div>
              <strong style={{ color: "var(--text-normal)" }}>Disable Sync (Local Only)</strong>
              <p style={{ color: "var(--text-muted)", fontSize: "13px", marginTop: "4px" }}>
                Keep all data local to this browser instance. No network requests will be made.
              </p>
            </div>
          </label>

          <label style={{
            display: "flex", 
            alignItems: "flex-start", 
            padding: "15px", 
            borderRadius: "8px", 
            border: `1px solid ${syncEngine === "chrome" ? "var(--accent-color)" : "var(--border-color)"}`, 
            backgroundColor: syncEngine === "chrome" ? "rgba(168, 85, 247, 0.05)" : "transparent",
            cursor: "pointer"
          }}>
            <input 
              type="radio" 
              name="syncEngine" 
              checked={syncEngine === "chrome"} 
              onChange={() => handleSaveSyncEngine("chrome")} 
              style={{ marginTop: "4px", marginRight: "12px" }}
            />
            <div>
              <strong style={{ color: "var(--text-normal)" }}>Chrome Profile Sync (chrome.storage.sync)</strong>
              <p style={{ color: "var(--text-muted)", fontSize: "13px", marginTop: "4px" }}>
                Syncs data transparently via your browser profile login. Simple and zero configuration. 
                <br />
                <span style={{ color: "#eab308" }}>⚠️ Strictly limited to 100 KB total size by Chrome. Recommended for settings and passwords only.</span>
              </p>
            </div>
          </label>

          <label style={{
            display: "flex", 
            alignItems: "flex-start", 
            padding: "15px", 
            borderRadius: "8px", 
            border: `1px solid ${syncEngine === "gdrive" ? "var(--accent-color)" : "var(--border-color)"}`, 
            backgroundColor: syncEngine === "gdrive" ? "rgba(168, 85, 247, 0.05)" : "transparent",
            cursor: "pointer"
          }}>
            <input 
              type="radio" 
              name="syncEngine" 
              checked={syncEngine === "gdrive"} 
              onChange={() => handleSaveSyncEngine("gdrive")} 
              style={{ marginTop: "4px", marginRight: "12px" }}
            />
            <div>
              <strong style={{ color: "var(--text-normal)" }}>Google Drive App Data Sync (Private Folder)</strong>
              <p style={{ color: "var(--text-muted)", fontSize: "13px", marginTop: "4px" }}>
                Store your notes and encrypted vault in your personal Google Drive account under a hidden application folder. 
                Provides virtually unlimited storage, highly secure.
              </p>
              {syncEngine === "gdrive" && (
                <div style={{ marginTop: "12px" }}>
                  <button 
                    type="button" 
                    className={`action-btn ${isGdriveAuthed ? "secondary" : "accent"}`}
                    onClick={handleGdriveAuth}
                  >
                    {isGdriveAuthed ? "Disconnect Google Account" : "Connect Google Account"}
                  </button>
                </div>
              )}
            </div>
          </label>
        </div>
      </section>

      <section className="settings-section" style={{ marginBottom: "40px" }}>
        <h2 style={{ fontSize: "20px", marginBottom: "15px", color: "var(--text-normal)" }}>Security Options</h2>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "15px 0", borderBottom: "1px solid var(--border-color)" }}>
          <div>
            <strong style={{ color: "var(--text-normal)" }}>Encrypt Synced Notes Client-Side</strong>
            <p style={{ color: "var(--text-muted)", fontSize: "13px", marginTop: "4px" }}>
              Encrypt notes with your vault master password before sending them to the cloud. Maximum privacy.
            </p>
          </div>
          <input 
            type="checkbox" 
            checked={encryptNotes} 
            onChange={handleToggleEncryption}
            style={{ width: "20px", height: "20px", accentColor: "var(--accent-color)", cursor: "pointer" }}
          />
        </div>
      </section>

      <section className="settings-section" style={{ marginBottom: "40px" }}>
        <h2 style={{ fontSize: "20px", marginBottom: "15px", color: "var(--text-normal)" }}>Sync Status & Actions</h2>
        <div style={{ backgroundColor: "var(--bg-card)", padding: "18px", borderRadius: "8px", border: "1px solid var(--border-color)", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div>
            <p style={{ color: "var(--text-normal)", fontSize: "14px" }}>
              <strong>Active Engine:</strong> {syncEngine === "none" ? "None (Local)" : syncEngine === "chrome" ? "Chrome Sync" : "Google Drive"}
            </p>
            <p style={{ color: "var(--text-muted)", fontSize: "13px", marginTop: "4px" }}>
              <strong>Last Synced:</strong> {syncStatus.lastSync > 0 ? new Date(syncStatus.lastSync).toLocaleString() : "Never"}
            </p>
            {syncStatus.error && (
              <p style={{ color: "#ef4444", fontSize: "13px", marginTop: "4px" }}>
                <strong>Error:</strong> {syncStatus.error}
              </p>
            )}
          </div>
          {syncEngine !== "none" && (
            <button type="button" className="action-btn accent" onClick={triggerManualSync}>
              Sync Now
            </button>
          )}
        </div>
      </section>

      <section className="settings-section">
        <h2 style={{ fontSize: "20px", marginBottom: "15px", color: "var(--text-normal)" }}>Local Storage Stats</h2>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "15px" }}>
          <div style={{ padding: "15px", border: "1px solid var(--border-color)", borderRadius: "8px", textAlign: "center" }}>
            <span style={{ fontSize: "24px", fontWeight: "bold", color: "var(--accent-color)", display: "block" }}>{stats.notesCount}</span>
            <span style={{ color: "var(--text-muted)", fontSize: "13px" }}>Notes Created</span>
          </div>
          <div style={{ padding: "15px", border: "1px solid var(--border-color)", borderRadius: "8px", textAlign: "center" }}>
            <span style={{ fontSize: "24px", fontWeight: "bold", color: "var(--accent-color)", display: "block" }}>{stats.secretsCount}</span>
            <span style={{ color: "var(--text-muted)", fontSize: "13px" }}>Secrets Stored</span>
          </div>
          <div style={{ padding: "15px", border: "1px solid var(--border-color)", borderRadius: "8px", textAlign: "center" }}>
            <span style={{ fontSize: "24px", fontWeight: "bold", color: "var(--accent-color)", display: "block" }}>
              {(stats.bytesUsed / 1024).toFixed(2)} KB
            </span>
            <span style={{ color: "var(--text-muted)", fontSize: "13px" }}>Estimated Storage</span>
          </div>
        </div>
      </section>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Options />
  </React.StrictMode>
);
