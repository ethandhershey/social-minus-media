"use client";

import { useState, useEffect } from "react";
import styles from "./alignment.module.css";
import { useRouter } from "next/navigation";

export default function AlignmentPage() {
  const [activities, setActivities] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();

  const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

  // 1. Load existing interests on mount
  useEffect(() => {
    async function loadInterests() {
      const token = sessionStorage.getItem("access_token");
      if (!token) {
        router.push("/");
        return;
      }

      try {
        const res = await fetch(`${baseUrl}/api/me/interests`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        if (res.ok) {
          const data = await res.json();
          // Assuming 'messages' is stored as { text: "..." } or similar
          if (data && data.messages && data.messages.text) {
            setActivities(data.messages.text);
          } else if (typeof data?.messages === "string") {
            setActivities(data.messages);
          }
        }
      } catch (err) {
        console.error("Failed to load interests:", err);
      } finally {
        setIsLoading(false);
      }
    }
    loadInterests();
  }, [baseUrl, router]);

  // 2. Handle PUT request to update
  async function handleSubmit(e) {
    e.preventDefault();
    const token = sessionStorage.getItem("access_token");
    if (!token || !activities.trim()) return;

    setIsSubmitting(true);

    try {
      const response = await fetch(`${baseUrl}/api/me/interests`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        // We wrap the paragraph in an object to satisfy the 'Value' type in Rust
        body: JSON.stringify({
          messages: { text: activities.trim() },
        }),
      });

      if (response.ok) {
        // Redirect back to profile or activities after successful save
        router.push("/profile");
      } else {
        throw new Error("Failed to update interests");
      }
    } catch (err) {
      alert("Error saving interests. Please try again.");
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  }

  if (isLoading) return <div className={styles.page}><main className={styles.main}>Loading...</main></div>;

  return (
    <div className={styles.page}>
      <main className={styles.main}>
        <div className={styles.card}>
          <h1 className={styles.heading}>What activities are you into?</h1>
          <p className={styles.hint}>
            Write a paragraph about the kinds of things you like to do. This helps us suggest activities that you'd actually be interested in.
          </p>

          <form className={styles.form} onSubmit={handleSubmit}>
            <label htmlFor="activities" className={styles.label}>
              Your activities
            </label>
            <textarea
              id="activities"
              className={styles.textarea}
              name="activities"
              rows={8}
              placeholder="e.g. I like chill board game nights, weekend hikes before noon, and trying new coffee shops with a small group..."
              value={activities}
              onChange={(e) => setActivities(e.target.value)}
              required
            />
            <button 
              type="submit" 
              className={styles.submit} 
              disabled={isSubmitting}
            >
              {isSubmitting ? "Saving..." : "Save Interests"}
            </button>
          </form>
        </div>
      </main>
    </div>
  );
}