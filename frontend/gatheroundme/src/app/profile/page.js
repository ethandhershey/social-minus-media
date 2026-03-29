"use client";

import Link from "next/link";
import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import pageStyles from "../page.module.css";
import styles from "./profile.module.css";

export default function ProfilePage() {
  const router = useRouter();
  
  // State for form fields
  const [username, setUsername] = useState("");
  const [bio, setBio] = useState("");
  const [email, setEmail] = useState("");
  const [hometown, setHometown] = useState("");
  
  // State for hidden/system data
  const [coords, setCoords] = useState({ lat: null, lng: null });
  const [locStatus, setLocStatus] = useState("idle");
  
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

  useEffect(() => {
    async function fetchProfile() {
      const token = sessionStorage.getItem("access_token");
      if (!token) {
        router.push("/");
        return;
      }

      try {
        const response = await fetch(`${baseUrl}/api/profile`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        if (response.ok) {
          const data = await response.json();
          setUsername(data.display_name || "");
          setBio(data.bio || "");
          setEmail(data.email || "");
          setHometown(data.city || "");
          // Store existing lat/lng so we don't send nulls later
          setCoords({ lat: data.latitude, lng: data.longitude });
        } else if (response.status === 401) {
          router.push("/");
        }
      } catch (err) {
        console.error("Failed to fetch profile:", err);
      } finally {
        setIsLoading(false);
      }
    }

    fetchProfile();
  }, [router, baseUrl]);

  const handleUpdateLocation = () => {
    setLocStatus("loading");
    if (!navigator.geolocation) {
      alert("Geolocation not supported");
      setLocStatus("error");
      return;
    }

    navigator.geolocation.getCurrentPosition(
      (pos) => {
        setCoords({ lat: pos.coords.latitude, lng: pos.coords.longitude });
        setLocStatus("success");
      },
      () => {
        setLocStatus("error");
        alert("Location access denied.");
      },
      { enableHighAccuracy: true, timeout: 5000 }
    );
  };

  async function handleSave(e) {
    e.preventDefault();
    const token = sessionStorage.getItem("access_token");
    if (!token) return;

    setIsSaving(true);

    const payload = {
      display_name: username.trim(),
      bio: bio.trim(),
      email: email.trim(),
      city: hometown.trim(),
      // Use existing coords from state so they don't override to null
      latitude: coords.lat,
      longitude: coords.lng,
      avatar_url: null, 
    };

    try {
      const response = await fetch(`${baseUrl}/api/profile`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) throw new Error("Update failed");
      alert("Profile updated!");
    } catch (err) {
      console.error(err);
      alert("Error saving changes.");
    } finally {
      setIsSaving(false);
    }
  }

  if (isLoading) return <div className={pageStyles.page}><main className={pageStyles.main}>Loading...</main></div>;

  return (
    <div className={pageStyles.page}>
      <header className={pageStyles.header}>
        <Link className={pageStyles.brand} href="/">
          <img src="/campfirelogo512.PNG" alt="logo" style={{ width: "1.75rem", marginRight: "0.35rem" }} />
          GatheRound
        </Link>
        <nav className={pageStyles.nav}>
          <Link className={pageStyles.btnGhost} href="/activities">Activities</Link>
        </nav>
      </header>

      <main className={`${pageStyles.main} ${styles.profileMain}`}>
        <h1 className={styles.username}>{username || "Your profile"}</h1>

        <form className={styles.form} onSubmit={handleSave}>
          <div className={styles.field}>
            <label className={styles.label}>Username</label>
            <input className={styles.input} value={username} onChange={(e) => setUsername(e.target.value)} required />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Email</label>
            <input className={styles.input} type="email" value={email} onChange={(e) => setEmail(e.target.value)} required />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Hometown</label>
            <input className={styles.input} value={hometown} onChange={(e) => setHometown(e.target.value)} placeholder="e.g. Austin, TX" />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Bio</label>
            <textarea className={styles.textarea} value={bio} onChange={(e) => setBio(e.target.value)} required />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Coordinates</label>
            <button 
              type="button" 
              className={styles.alignmentLink} 
              onClick={handleUpdateLocation}
              style={{ 
                width: '100%', 
                background: locStatus === 'success' ? '#e8f5e9' : 'transparent' 
              }}
            >
              {locStatus === "success" ? "✓ Location Updated" : "Update GPS Location"}
            </button>
            <p className={styles.hint}>
              Current: {coords.lat ? `${coords.lat.toFixed(3)}, ${coords.lng.toFixed(3)}` : "Not set"}
            </p>
          </div>

          <div className={styles.saveRow}>
            <button type="submit" className={styles.saveBtn} disabled={isSaving}>
              {isSaving ? "Saving..." : "Save changes"}
            </button>
          </div>
        </form>

        <section className={styles.alignmentBlock}>
          <Link className={styles.alignmentLink} href="/alignment">Redo alignment test</Link>
        </section>
      </main>
    </div>
  );
}