"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import styles from "./signup.module.css";

export default function SignupPage() {
  const router = useRouter();
  const [formData, setFormData] = useState({ 
    username: "", 
    email: "", 
    bio: "", 
    hometown: "" 
  });
  const [location, setLocation] = useState(null);
  const [locStatus, setLocStatus] = useState("idle");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleLocation = () => {
    setLocStatus("loading");
    if (!navigator.geolocation) {
      alert("Geolocation is not supported by your browser");
      setLocStatus("error");
      return;
    }

    navigator.geolocation.getCurrentPosition(
      (pos) => {
        setLocation({ lat: pos.coords.latitude, lng: pos.coords.longitude });
        setLocStatus("success");
      },
      () => {
        setLocStatus("error");
        alert("Please allow location access to continue.");
      },
      { enableHighAccuracy: true, timeout: 5000, maximumAge: 0 }
    );
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    // 1. HARD VALIDATION: Ensure we have the critical data
    if (!location) return alert("Please allow location access first.");
    if (!formData.username.trim() || !formData.email.trim()) {
      return alert("Username and Email are required to complete signup.");
    }

    setIsSubmitting(true);

    // 2. CONSTRUCT PAYLOAD: Explicitly mapping to match your backend DTO
    const payload = {
      display_name: formData.username.trim(), 
      email: formData.email.trim(),
      avatar_url: null,
      bio: formData.bio.trim() || null,
      city: formData.hometown.trim() || null,
      latitude: location.lat,
      longitude: location.lng,
    };

    try {
      const token = sessionStorage.getItem("access_token");
      const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
      
      if (!token) {
        throw new Error("No session token found. Please sign in again.");
      }

      console.log("Submitting Profile Payload:", payload); // Debug check

      const response = await fetch(`${baseUrl}/api/profile`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const errorText = await response.text(); 
        const errorData = errorText ? JSON.parse(errorText) : {};
        throw new Error(errorData.message || `Server error: ${response.status}`);
      }

      // 3. SUCCESS REDIRECT
      // Pushing to alignment first to ensure the backend has processed the profile
      router.push("/alignment");
      
    } catch (err) {
      console.error("Profile Update Error:", err);
      alert(err.message);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className={styles.signupWrapper}>
      <main className={styles.profileMain}>
        <h1 className={styles.username}>Finish your profile</h1>
        <p className={styles.hint}>Almost there! We just need a few more details.</p>

        <form className={styles.form} onSubmit={handleSubmit}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="username">Username</label>
            <input
              id="username"
              type="text"
              className={styles.input}
              placeholder="Pick a unique name"
              required
              value={formData.username}
              onChange={(e) => setFormData({ ...formData, username: e.target.value })}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="email">Email Address</label>
            <input
              id="email"
              type="email"
              className={styles.input}
              placeholder="your@email.com"
              required
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="hometown">Hometown (City)</label>
            <input
              id="hometown"
              type="text"
              className={styles.input}
              placeholder="e.g. Austin, TX"
              required
              value={formData.hometown}
              onChange={(e) => setFormData({ ...formData, hometown: e.target.value })}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="bio">Bio</label>
            <textarea
              id="bio"
              className={styles.textarea}
              placeholder="Tell people about yourself..."
              required
              value={formData.bio}
              onChange={(e) => setFormData({ ...formData, bio: e.target.value })}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Location Discovery</label>
            <button 
              type="button" 
              className={styles.alignmentLink} 
              onClick={handleLocation}
              style={{ 
                width: '100%', 
                background: locStatus === 'success' ? '#e8f5e9' : 'transparent',
                borderColor: locStatus === 'success' ? '#7dae8a' : '#ddd'
              }}
            >
              {locStatus === "loading" && "Fetching coordinates..."}
              {locStatus === "success" && "✓ Location Shared"}
              {locStatus === "error" && "✕ Access Denied"}
              {locStatus === "idle" && "Enable Location"}
            </button>
          </div>

          <div className={styles.saveRow}>
            <button 
              type="submit" 
              className={styles.saveBtn} 
              disabled={isSubmitting || locStatus !== 'success'}
            >
              {isSubmitting ? "Saving..." : "Finish Profile & Explore"}
            </button>
          </div>
        </form>
      </main>
    </div>
  );
}