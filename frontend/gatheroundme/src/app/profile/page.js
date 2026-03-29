"use client";

import Link from "next/link";
import { useState } from "react";
import pageStyles from "../page.module.css";
import styles from "./profile.module.css";

// TODO: replace with GET /api/me or GET /api/profile when backend exists
const PLACEHOLDER_PROFILE = {
  username: "demo_user",
  bio: "I like weekend hikes and chill board game nights. Always up for coffee before noon.",
  email: "you@example.com",
};

export default function ProfilePage() {
  const [username, setUsername] = useState(PLACEHOLDER_PROFILE.username);
  const [bio, setBio] = useState(PLACEHOLDER_PROFILE.bio);
  const [email, setEmail] = useState(PLACEHOLDER_PROFILE.email);

  function handleSave(e) {
    e.preventDefault();
    // PATCH /api/profile  (or PUT /api/me)
    // Content-Type: application/json
    // Body: { username, bio, email }
    // await fetch("/api/profile", {
    //   method: "PATCH",
    //   headers: { "Content-Type": "application/json" },
    //   body: JSON.stringify({ username: username.trim(), bio: bio.trim(), email: email.trim() }),
    // });
  }

  return (
    <div className={pageStyles.page}>
      <header className={pageStyles.header}>
        <Link className={pageStyles.brand} href="/">
          GatheRound
        </Link>
        <nav className={pageStyles.nav}>
          <Link className={pageStyles.btnGhost} href="/activities">
            Activities
          </Link>
        </nav>
      </header>

      <main className={`${pageStyles.main} ${styles.profileMain}`}>
        <h1 className={styles.username}>{username.trim() || "Your profile"}</h1>

        <form className={styles.form} onSubmit={handleSave}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="profile-username">
              Username
            </label>
            <input
              id="profile-username"
              className={styles.input}
              type="text"
              autoComplete="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="profile-bio">
              Bio
            </label>
            <textarea
              id="profile-bio"
              className={styles.textarea}
              name="bio"
              value={bio}
              onChange={(e) => setBio(e.target.value)}
            />
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="profile-email">
              Email
            </label>
            <input
              id="profile-email"
              className={styles.input}
              type="email"
              autoComplete="email"
              inputMode="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>

          <div className={styles.saveRow}>
            <button type="submit" className={styles.saveBtn}>
              Save changes
            </button>
            <p className={styles.hint}>
              Placeholder — wire Save to your API when ready.
            </p>
          </div>
        </form>

        <section className={styles.alignmentBlock} aria-labelledby="alignment-heading">
          <p id="alignment-heading" className={styles.alignmentLabel}>
            Activity alignment
          </p>
          <Link className={styles.alignmentLink} href="/alignment">
            Redo alignment test
          </Link>
        </section>
      </main>

      <footer className={pageStyles.footer}>
        <p>© {new Date().getFullYear()} GatheRound</p>
      </footer>
    </div>
  );
}