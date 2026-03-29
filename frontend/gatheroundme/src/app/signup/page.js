"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import styles from "./signup.module.css";

export default function SignupPage() {
  const router = useRouter();
  const [formData, setFormData] = useState({ bio: "", hometown: "" });
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
    if (!location) return alert("Please allow location access first.");

    setIsSubmitting(true);

    const payload = {
      bio: formData.bio,
      city: formData.hometown,
      latitude: location.lat,
      longitude: location.lng,
      // sub: userSub, // This would come from your Zitadel token state
    };

    console.log("Submitting to backend:", payload);

    /* // BACKEND POST LOGIC (Commented out for now)
    try {
      const response = await fetch('/api/users/profile', {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionStorage.getItem('access_token')}`
        },
        body: JSON.stringify(payload)
      });
      if (response.ok) router.push('/activities');
    } catch (err) {
      console.error(err);
    }
    */

    // Simulate successful POST and redirect
    setTimeout(() => {
      router.push("/alignment");
    }, 1000);
  };

  return (
    <div className={styles.signupWrapper}>
      <main className={styles.profileMain}>
        <h1 className={styles.username}>Finish your profile</h1>
        <p className={styles.hint}>Almost there! We just need a few more details.</p>

        <form className={styles.form} onSubmit={handleSubmit}>
          <div className={styles.field}>
            <label className={styles.label}>Hometown (City)</label>
            <input
              type="text"
              className={styles.input}
              placeholder="e.g. Austin, TX"
              required
              value={formData.hometown}
              onChange={(e) => setFormData({ ...formData, hometown: e.target.value })}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label}>Bio</label>
            <textarea
              className={styles.textarea}
              placeholder="Tell people about yourself and what you like to do..."
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
                background: locStatus === 'success' ? 'var(--green-light, #e8f5e9)' : 'transparent',
                borderColor: locStatus === 'success' ? 'var(--green, #7dae8a)' : ''
              }}
            >
              {locStatus === "loading" && "Fetching coordinates..."}
              {locStatus === "success" && "✓ Location Shared"}
              {locStatus === "error" && "✕ Access Denied"}
              {locStatus === "idle" && "Enable Location"}
            </button>
            <p className={styles.hint}>Required to find activities in your area.</p>
          </div>

          <div className={styles.saveRow}>
            <button type="submit" className={styles.saveBtn} disabled={isSubmitting || !location}>
              {isSubmitting ? "Saving..." : "Finish Profile & Explore"}
            </button>
          </div>
        </form>
      </main>
    </div>
  );
}