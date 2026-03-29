"use client";

import { useState } from "react";
import styles from "./alignment.module.css";
import Link from "next/link";

export default function AlignmentPage() {
  const [activities, setActivities] = useState("");
  const [submitted, setSubmitted] = useState(false);

  function handleSubmit(e) {
    e.preventDefault();
    const payload = { activitiesParagraph: activities.trim() };
    if (!payload.activitiesParagraph) return;

    // TODO: POST to API when backend is ready, e.g.:
    // await fetch("/api/alignment", {
    //   method: "POST",
    //   headers: { "Content-Type": "application/json" },
    //   body: JSON.stringify(payload),
    // });

    setSubmitted(true);
  }

  return (
    <div className={styles.page}>
      <main className={styles.main}>
        <div className={styles.card}>
          <h1 className={styles.heading}>What activities are you into?</h1>
          <p className={styles.hint}>
            Write a paragraph about the kinds of things you like to do. This helps us suggest activities that you'd actually be interested in.
          </p>

          {submitted ? (
            <p className={styles.success} role="status">
              Thanks—we&apos;ll use this to build your profile when the backend is
              connected.
            </p>
          ) : (
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
              <Link href="/profile">
                <button type="submit" className={styles.submit}>
                  Submit
                </button>
              </Link>
            </form>
          )}
        </div>
      </main>
    </div>
  );
}